use crate::models::{CreateQuizRequest, Question, Quiz};
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use serde_json::{json, Value as JsonValue};
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};

/// List all quizzes
pub async fn list_quizzes(pool: web::Data<SqlitePool>) -> impl Responder {
    let pool = pool.get_ref();

    let rows =
        match sqlx::query("SELECT id, title, description FROM quizzes ORDER BY created_at DESC")
            .fetch_all(pool)
            .await
        {
            Ok(r) => r,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to fetch quizzes"
                }));
            }
        };

    let mut quizzes = Vec::new();
    for row in rows {
        let quiz_id: i64 = row.try_get("id").unwrap_or(0);

        let questions_rows = match sqlx::query(
            "SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = ?",
        )
        .bind(quiz_id)
        .fetch_all(pool)
        .await
        {
            Ok(r) => r,
            Err(_) => Vec::new(),
        };

        let questions: Vec<Question> = questions_rows
            .into_iter()
            .map(|r| {
                let id: i64 = r.try_get("id").unwrap_or(0);
                let text: String = r.try_get("text").unwrap_or_default();
                let options_text: String =
                    r.try_get("options").unwrap_or_else(|_| "[]".to_string());
                let options: Vec<String> = serde_json::from_str(&options_text).unwrap_or_default();
                let correct_answer: i64 = r.try_get("correct_answer").unwrap_or(0);

                Question {
                    id: Some(id as i32),
                    text,
                    options,
                    correct_answer: correct_answer as i32,
                }
            })
            .collect();

        let title: String = row.try_get("title").unwrap_or_default();
        let description: Option<String> = row.try_get("description").ok();

        quizzes.push(Quiz {
            id: quiz_id as i32,
            title,
            description,
            questions,
        });
    }

    HttpResponse::Ok().json(quizzes)
}

/// Get a single quiz
pub async fn get_quiz(pool: web::Data<SqlitePool>, quiz_id: web::Path<i32>) -> impl Responder {
    let pool = pool.get_ref();
    let id = quiz_id.into_inner() as i64;

    let quiz_row = match sqlx::query("SELECT id, title, description FROM quizzes WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::NotFound().json(json!({"error": "Quiz not found"})),
    };

    let questions_rows = match sqlx::query(
        "SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = ?",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to fetch questions"}))
        }
    };

    let questions: Vec<Question> = questions_rows
        .into_iter()
        .map(|r| {
            let id: i64 = r.try_get("id").unwrap_or(0);
            let text: String = r.try_get("text").unwrap_or_default();
            let options_text: String = r.try_get("options").unwrap_or_else(|_| "[]".to_string());
            let options: Vec<String> = serde_json::from_str(&options_text).unwrap_or_default();
            let correct_answer: i64 = r.try_get("correct_answer").unwrap_or(0);

            Question {
                id: Some(id as i32),
                text,
                options,
                correct_answer: correct_answer as i32,
            }
        })
        .collect();

    let quiz = Quiz {
        id: quiz_row.try_get::<i64, _>("id").unwrap_or(0) as i32,
        title: quiz_row.try_get::<String, _>("title").unwrap_or_default(),
        description: quiz_row
            .try_get::<Option<String>, _>("description")
            .ok()
            .flatten(),
        questions,
    };

    HttpResponse::Ok().json(quiz)
}

/// Create a quiz (and its questions)
pub async fn create_quiz(
    pool: web::Data<SqlitePool>,
    quiz_data: web::Json<CreateQuizRequest>,
) -> impl Responder {
    let pool = pool.get_ref();

    let res = match sqlx::query("INSERT INTO quizzes (title, description) VALUES (?, ?)")
        .bind(&quiz_data.title)
        .bind(&quiz_data.description)
        .execute(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error":"Failed to create quiz"}))
        }
    };

    let quiz_id = res.last_insert_rowid() as i32;

    for question in &quiz_data.questions {
        let options_json = match serde_json::to_string(&question.options) {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(json!({"error":"Failed to serialize question options"}))
            }
        };

        if let Err(_) = sqlx::query(
            "INSERT INTO questions (quiz_id, text, options, correct_answer) VALUES (?, ?, ?, ?)",
        )
        .bind(quiz_id as i64)
        .bind(&question.text)
        .bind(&options_json)
        .bind(question.correct_answer as i64)
        .execute(pool)
        .await
        {
            return HttpResponse::InternalServerError()
                .json(json!({"error":"Failed to create questions"}));
        }
    }

    HttpResponse::Created().json(json!({"id": quiz_id, "message": "Quiz created successfully"}))
}

/// Create a test based on selection criteria and store it in `tests` table.
/// Minimal, production-safe behavior without debug output.
pub async fn create_test(
    pool: web::Data<SqlitePool>,
    payload: web::Json<JsonValue>,
) -> impl Responder {
    let pool = pool.get_ref();
    let body = payload.into_inner();

    let level_label = body.get("level").and_then(|v| v.as_str()).unwrap_or("n4");
    let mode = body
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("chapter");
    let chapters = body.get("chapters").and_then(|v| v.as_array()).cloned();
    let range = body.get("range").cloned();
    let num_questions = body.get("numQuestions").and_then(|v| v.as_i64());

    let level_map = vec![("n5", 1), ("n4", 2), ("n3", 3), ("n2", 4), ("n1", 5)]
        .into_iter()
        .collect::<std::collections::HashMap<&str, i64>>();
    let level_id_opt = level_map.get(level_label).cloned();

    let mut where_clauses: Vec<String> = Vec::new();
    let mut binds: Vec<JsonValue> = Vec::new();

    if let Some(lid) = level_id_opt {
        where_clauses.push("level = ?".to_string());
        binds.push(JsonValue::from(lid));
    }

    if mode == "chapter" {
        if let Some(chs) = &chapters {
            let nums: Vec<i64> = chs.iter().filter_map(|v| v.as_i64()).collect();
            if !nums.is_empty() {
                let placeholders = vec!["?"; nums.len()].join(", ");
                where_clauses.push(format!("chapter IN ({})", placeholders));
                for n in nums {
                    binds.push(JsonValue::from(n));
                }
            }
        }
    } else if mode == "range" {
        if let Some(r) = &range {
            if let (Some(s), Some(e)) = (
                r.get("start").and_then(|v| v.as_i64()),
                r.get("end").and_then(|v| v.as_i64()),
            ) {
                where_clauses.push("entry_id BETWEEN ? AND ?".to_string());
                binds.push(JsonValue::from(s));
                binds.push(JsonValue::from(e));
            }
        }
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    let limit_clause = if let Some(nq) = num_questions {
        format!("LIMIT {}", nq)
    } else {
        "".to_string()
    };
    let order_clause = "ORDER BY RANDOM()";

    // Ensure tests table exists
    if let Err(_) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS tests (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT, questions TEXT, created_at TEXT DEFAULT (datetime('now')))"
    ).execute(pool).await {
        return HttpResponse::InternalServerError().json(json!({"error":"Failed to prepare tests storage"}));
    }

    let query_sql = format!("SELECT id, entry_id, q_type, prompt, options, correct_index, correct_answer FROM questions {} {} {}", where_sql, order_clause, limit_clause);

    // Primary attempt
    let mut q = sqlx::query(&query_sql);
    for b in binds.iter() {
        match b {
            JsonValue::Number(num) => {
                if let Some(i) = num.as_i64() {
                    q = q.bind(i);
                } else if let Some(u) = num.as_u64() {
                    q = q.bind(u as i64);
                } else if let Some(f) = num.as_f64() {
                    q = q.bind(f);
                } else {
                    q = q.bind(num.to_string());
                }
            }
            JsonValue::String(s) => {
                q = q.bind(s.as_str());
            }
            JsonValue::Bool(bv) => {
                q = q.bind(if *bv { 1i64 } else { 0i64 });
            }
            JsonValue::Null => {
                q = q.bind(None::<i64>);
            }
            other => {
                q = q.bind(other.to_string());
            }
        }
    }

    let mut rows: Vec<SqliteRow> = match q.fetch_all(pool).await {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error":"Database error"}))
        }
    };

    // Simple fallback: if nothing selected, try global selection without filters
    if rows.is_empty() {
        let query_any = format!("SELECT id, entry_id, q_type, prompt, options, correct_index, correct_answer FROM questions {} {}", order_clause, limit_clause);
        let mut q_any = sqlx::query(&query_any);
        match q_any.fetch_all(pool).await {
            Ok(r2) => rows = r2,
            Err(_) => {
                return HttpResponse::InternalServerError().json(json!({"error":"Database error"}))
            }
        }
    }

    if rows.is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"error":"No questions matched the selection"}));
    }

    let mut test_questions: Vec<JsonValue> = Vec::new();
    for r in rows {
        let qid: i64 = r.try_get("id").unwrap_or(0);
        let prompt: String = r.try_get("prompt").unwrap_or_default();
        let options_text: String = r.try_get("options").unwrap_or_else(|_| "[]".to_string());
        let options: Vec<String> = serde_json::from_str(&options_text).unwrap_or_default();
        let correct_index: Option<i64> = r.try_get::<i64, _>("correct_index").ok();
        let correct_answer_idx: Option<i64> = r.try_get::<i64, _>("correct_answer").ok();
        let correct = correct_index.or(correct_answer_idx).unwrap_or(0);

        test_questions.push(json!({
            "id": qid,
            "text": prompt,
            "options": options,
            "correct_index": correct
        }));
    }

    let title = format!("Test - {} - {}", level_label, Utc::now().to_rfc3339());
    let questions_json = match serde_json::to_string(&test_questions) {
        Ok(s) => s,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error":"Failed to serialize test"}))
        }
    };

    let res = match sqlx::query("INSERT INTO tests (title, questions) VALUES (?, ?)")
        .bind(&title)
        .bind(&questions_json)
        .execute(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::InternalServerError().json(json!({"error":"Failed to save test"}))
        }
    };

    let test_id = res.last_insert_rowid();
    let redirect = format!("/test/{}", test_id);

    HttpResponse::Created().json(json!({"id": test_id, "redirect": redirect}))
}

/// Get a generated test by id
pub async fn get_test(pool: web::Data<SqlitePool>, test_id: web::Path<i64>) -> impl Responder {
    let pool = pool.get_ref();
    let id = test_id.into_inner();

    let row = match sqlx::query("SELECT id, title, questions, created_at FROM tests WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => return HttpResponse::NotFound().json(json!({"error": "Test not found"})),
    };

    let questions_text: String = row
        .try_get("questions")
        .unwrap_or_else(|_| "[]".to_string());
    let questions: JsonValue = match serde_json::from_str(&questions_text) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error":"Failed to parse test data"}))
        }
    };

    let title: String = row.try_get("title").unwrap_or_default();

    HttpResponse::Ok().json(json!({
        "id": id,
        "title": title,
        "questions": questions
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/quizzes", web::get().to(list_quizzes))
            .route("/quizzes", web::post().to(create_quiz))
            .route("/quizzes/{id}", web::get().to(get_quiz))
            .route("/tests", web::post().to(create_test))
            .route("/tests/{id}", web::get().to(get_test)),
    );
}
