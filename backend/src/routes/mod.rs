use actix_web::{web, HttpResponse, Responder};
use sqlx::{SqlitePool, Row};
use sqlx::sqlite::SqliteRow;
use serde_json::json;
use serde_json::Value as JsonValue;
use crate::models::{CreateQuizRequest, Quiz, Question};

/// List quizzes (unchanged)
pub async fn list_quizzes(pool: web::Data<SqlitePool>) -> impl Responder {
    let pool = pool.get_ref();

    let rows = match sqlx::query("SELECT id, title, description FROM quizzes ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            // Log full error on server side for debugging
            eprintln!("Database error when listing quizzes: {:?}", e);
            // Return an informative error payload (message + details)
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch quizzes",
                "details": format!("{}", e)
            }));
        }
    };

    let mut quizzes = Vec::new();
    for row in rows {
        let quiz_id: i64 = row.try_get("id").unwrap_or(0);

        // Get questions for each quiz
        let questions_rows = match sqlx::query("SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = ?")
            .bind(quiz_id)
            .fetch_all(pool)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Database error fetching questions for quiz {}: {}", quiz_id, e);
                Vec::new()
            }
        };

        let questions: Vec<Question> = questions_rows
            .into_iter()
            .map(|r| {
                let id: i64 = r.try_get("id").unwrap_or(0);
                let text: String = r.try_get("text").unwrap_or_default();
                // options stored as JSON in TEXT column
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

/// Get a single quiz (unchanged)
pub async fn get_quiz(
    pool: web::Data<SqlitePool>,
    quiz_id: web::Path<i32>,
) -> impl Responder {
    let pool = pool.get_ref();
    let id = quiz_id.into_inner() as i64;

    let quiz_row = match sqlx::query("SELECT id, title, description FROM quizzes WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(row) => row,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({
                "error": "Quiz not found"
            }));
        }
    };

    let questions_rows = match sqlx::query("SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = ?")
        .bind(id)
        .fetch_all(pool)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            // Log the DB error and return richer JSON so clients can surface actionable info
            eprintln!("Database error when fetching questions for quiz {}: {:?}", id, e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to fetch questions",
                "details": format!("{}", e)
            }));
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
        description: quiz_row.try_get::<Option<String>, _>("description").ok().flatten(),
        questions,
    };

    HttpResponse::Ok().json(quiz)
}

/// Create quiz (unchanged)
pub async fn create_quiz(
    pool: web::Data<SqlitePool>,
    quiz_data: web::Json<CreateQuizRequest>,
) -> impl Responder {
    let pool = pool.get_ref();

    // Insert quiz
    let res = match sqlx::query("INSERT INTO quizzes (title, description) VALUES (?, ?)")
        .bind(&quiz_data.title)
        .bind(&quiz_data.description)
        .execute(pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Database error inserting quiz: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create quiz",
                "details": format!("{}", e)
            }));
        }
    };

    // For SQLite, last_insert_rowid is available on the result
    let quiz_id = res.last_insert_rowid() as i32;

    // Insert questions
    for question in &quiz_data.questions {
        let options_json = match serde_json::to_string(&question.options) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize options: {:?}", e);
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to serialize question options",
                    "details": format!("{}", e)
                }));
            }
        };

        if let Err(e) = sqlx::query("INSERT INTO questions (quiz_id, text, options, correct_answer) VALUES (?, ?, ?, ?)")
            .bind(quiz_id as i64)
            .bind(&question.text)
            .bind(&options_json)
            .bind(question.correct_answer as i64)
            .execute(pool)
            .await
        {
            eprintln!("Database error inserting question: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": "Failed to create questions",
                "details": format!("{}", e)
            }));
        }
    }

    HttpResponse::Created().json(json!({
        "id": quiz_id,
        "message": "Quiz created successfully"
    }))
}

/// Create a test based on selection criteria and store it in `tests` table.
/// Expects JSON payload:
/// {
///   "level": "n4",               // optional, one of n5..n1
///   "mode": "chapter" | "range", // 'chapter' or 'range'
///   "chapters": [1,2,3],         // when mode=='chapter'
///   "range": { "start": 1, "end": 100 }, // when mode=='range'
///   "numQuestions": 20           // optional limit
/// }
pub async fn create_test(
    pool: web::Data<SqlitePool>,
    payload: web::Json<JsonValue>,
) -> impl Responder {
    let pool = pool.get_ref();

    // parse payload
    let body = payload.into_inner();
    // Debug log the incoming payload to help troubleshooting
    match serde_json::to_string_pretty(&body) {
        Ok(s) => eprintln!("create_test payload:\n{}", s),
        Err(_) => eprintln!("create_test payload: <unserializable>"),
    }

    let level_label = body.get("level").and_then(|v| v.as_str()).unwrap_or("n4");
    let mode = body.get("mode").and_then(|v| v.as_str()).unwrap_or("chapter");
    let chapters = body.get("chapters").and_then(|v| v.as_array()).cloned();
    let range = body.get("range").cloned();
    let num_questions = body.get("numQuestions").and_then(|v| v.as_i64());

    // map level label to numeric if questions.level stores numeric ids (the apply script used 1->n5..5->n1)
    let level_map = vec![("n5", 1), ("n4", 2), ("n3", 3), ("n2", 4), ("n1", 5)]
        .into_iter()
        .collect::<std::collections::HashMap<&str, i64>>();
    let level_id_opt = level_map.get(level_label).cloned();

    // Build WHERE clauses
    let mut where_clauses: Vec<String> = Vec::new();
    // Collect dynamic bind values as JSON values so we can inspect types at bind time
    // and bind concrete Rust types (i64, f64, &str, etc.) to sqlx queries.
    let mut binds: Vec<JsonValue> = Vec::new();

    if let Some(lid) = level_id_opt {
        where_clauses.push("level = ?".to_string());
        binds.push(JsonValue::from(lid));
    }

    if mode == "chapter" {
        if let Some(chs) = &chapters {
            let nums: Vec<i64> = chs.iter().filter_map(|v| v.as_i64()).collect();
            if !nums.is_empty() {
                // build IN clause with placeholders
                let placeholders = vec!["?"; nums.len()].join(", ");
                where_clauses.push(format!("chapter IN ({})", placeholders));
                for n in nums {
                    binds.push(JsonValue::from(n));
                }
            }
        }
    } else if mode == "range" {
        if let Some(r) = &range {
            if let (Some(s), Some(e)) = (r.get("start").and_then(|v| v.as_i64()), r.get("end").and_then(|v| v.as_i64())) {
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

    // Limit / randomize
    let limit_clause = if let Some(nq) = num_questions { format!("LIMIT {}", nq) } else { "".to_string() };
    let order_clause = "ORDER BY RANDOM()";

    // Ensure tests table exists
    if let Err(e) = sqlx::query(
        "CREATE TABLE IF NOT EXISTS tests (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT, questions TEXT, created_at TEXT DEFAULT (datetime('now')))"
    ).execute(pool).await {
        eprintln!("Failed to ensure tests table: {}", e);
        return HttpResponse::InternalServerError().json(json!({"error":"Failed to prepare tests storage"}));
    }

    // Build query string
    let query_sql = format!("SELECT id, entry_id, q_type, prompt, options, correct_index, correct_answer FROM questions {} {} {}", where_sql, order_clause, limit_clause);

    // Debug: log the SQL and bind values before executing
    eprintln!("create_test SQL: {}", query_sql);
    match serde_json::to_string(&binds) {
        Ok(s) => eprintln!("create_test binds: {}", s),
        Err(_) => eprintln!("create_test binds: <unserializable>"),
    }

    // Primary attempt: bind values inline
    let mut q_primary = sqlx::query(&query_sql);
    for b in binds.iter() {
        match b {
            JsonValue::Number(num) => {
                if let Some(i) = num.as_i64() {
                    q_primary = q_primary.bind(i);
                } else if let Some(u) = num.as_u64() {
                    q_primary = q_primary.bind(u as i64);
                } else if let Some(f) = num.as_f64() {
                    q_primary = q_primary.bind(f);
                } else {
                    q_primary = q_primary.bind(num.to_string());
                }
            }
            JsonValue::String(s) => {
                q_primary = q_primary.bind(s.as_str());
            }
            JsonValue::Bool(bv) => {
                q_primary = q_primary.bind(if *bv { 1i64 } else { 0i64 });
            }
            JsonValue::Null => {
                q_primary = q_primary.bind(None::<i64>);
            }
            other => {
                q_primary = q_primary.bind(other.to_string());
            }
        }
    }

    let mut rows: Vec<SqliteRow> = match q_primary.fetch_all(pool).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("DB error selecting questions for test: {}; SQL: {}; binds: {:?}", e, query_sql, binds);
            return HttpResponse::InternalServerError().json(json!({"error": format!("Database error: {}", e.to_string())}));
        }
    };

    // If primary returned no rows, attempt fallbacks progressively
    if rows.is_empty() {
        eprintln!("create_test: primary query returned 0 rows. Attempting fallbacks...");

        // 1) If mode == chapter and chapters specified, try join with entries table (use entries.chapter)
        if mode == "chapter" {
            if let Some(chs) = &chapters {
                let nums: Vec<i64> = chs.iter().filter_map(|v| v.as_i64()).collect();
                if !nums.is_empty() {
                    let placeholders = vec!["?"; nums.len()].join(", ");
                    // preserve level filter if present
                    let mut join_clauses: Vec<String> = Vec::new();
                    let mut join_binds: Vec<JsonValue> = Vec::new();
                    if let Some(lid) = level_id_opt {
                        join_clauses.push("q.level = ?".to_string());
                        join_binds.push(JsonValue::from(lid));
                    }
                    join_clauses.push(format!("e.chapter IN ({})", placeholders));
                    for n in nums.iter() {
                        join_binds.push(JsonValue::from(*n));
                    }
                    let where_join = format!("WHERE {}", join_clauses.join(" AND "));
                    let query_join = format!("SELECT q.id, q.entry_id, q.q_type, q.prompt, q.options, q.correct_index, q.correct_answer FROM questions q JOIN entries e ON q.entry_id = e.id {} {} {}", where_join, order_clause, limit_clause);

                    eprintln!("create_test fallback #1 (join with entries) SQL: {}", query_join);
                    match serde_json::to_string(&join_binds) {
                        Ok(s) => eprintln!("create_test fallback #1 binds: {}", s),
                        Err(_) => eprintln!("create_test fallback #1 binds: <unserializable>"),
                    }

                    let mut q_join = sqlx::query(&query_join);
                    for b in join_binds.iter() {
                        match b {
                            JsonValue::Number(num) => {
                                if let Some(i) = num.as_i64() {
                                    q_join = q_join.bind(i);
                                } else if let Some(u) = num.as_u64() {
                                    q_join = q_join.bind(u as i64);
                                } else if let Some(f) = num.as_f64() {
                                    q_join = q_join.bind(f);
                                } else {
                                    q_join = q_join.bind(num.to_string());
                                }
                            }
                            JsonValue::String(s) => {
                                q_join = q_join.bind(s.as_str());
                            }
                            JsonValue::Bool(bv) => {
                                q_join = q_join.bind(if *bv { 1i64 } else { 0i64 });
                            }
                            JsonValue::Null => {
                                q_join = q_join.bind(None::<i64>);
                            }
                            other => {
                                q_join = q_join.bind(other.to_string());
                            }
                        }
                    }

                    match q_join.fetch_all(pool).await {
                        Ok(r2) => {
                            if !r2.is_empty() {
                                eprintln!("create_test fallback #1 succeeded with {} rows", r2.len());
                                rows = r2;
                            } else {
                                eprintln!("create_test fallback #1 returned 0 rows");
                            }
                        }
                        Err(e) => {
                            eprintln!("create_test fallback #1 DB error: {}", e);
                        }
                    }
                }
            }
        }

        // 2) If still empty: try dropping the level filter (i.e., ignore level constraint)
        if rows.is_empty() {
            eprintln!("create_test fallback #2: trying without level filter");
            // build where_clauses without "level = ?" parts
            let mut clauses_no_level: Vec<String> = where_clauses.iter().cloned().filter(|c| c.trim() != "level = ?").collect();
            let mut binds_no_level = binds.clone();
            // remove first occurrence of level value if present
            if let Some(lid) = level_id_opt {
                if let Some(pos) = binds_no_level.iter().position(|b| *b == JsonValue::from(lid)) {
                    binds_no_level.remove(pos);
                }
            }
            let where_no_level_sql = if clauses_no_level.is_empty() { String::new() } else { format!("WHERE {}", clauses_no_level.join(" AND ")) };
            let query_no_level = format!("SELECT id, entry_id, q_type, prompt, options, correct_index, correct_answer FROM questions {} {} {}", where_no_level_sql, order_clause, limit_clause);

            eprintln!("create_test fallback #2 SQL: {}", query_no_level);
            match serde_json::to_string(&binds_no_level) {
                Ok(s) => eprintln!("create_test fallback #2 binds: {}", s),
                Err(_) => eprintln!("create_test fallback #2 binds: <unserializable>"),
            }

            let mut q_no_level = sqlx::query(&query_no_level);
            for b in binds_no_level.iter() {
                match b {
                    JsonValue::Number(num) => {
                        if let Some(i) = num.as_i64() {
                            q_no_level = q_no_level.bind(i);
                        } else if let Some(u) = num.as_u64() {
                            q_no_level = q_no_level.bind(u as i64);
                        } else if let Some(f) = num.as_f64() {
                            q_no_level = q_no_level.bind(f);
                        } else {
                            q_no_level = q_no_level.bind(num.to_string());
                        }
                    }
                    JsonValue::String(s) => {
                        q_no_level = q_no_level.bind(s.as_str());
                    }
                    JsonValue::Bool(bv) => {
                        q_no_level = q_no_level.bind(if *bv { 1i64 } else { 0i64 });
                    }
                    JsonValue::Null => {
                        q_no_level = q_no_level.bind(None::<i64>);
                    }
                    other => {
                        q_no_level = q_no_level.bind(other.to_string());
                    }
                }
            }

            match q_no_level.fetch_all(pool).await {
                Ok(r2) => {
                    if !r2.is_empty() {
                        eprintln!("create_test fallback #2 succeeded with {} rows", r2.len());
                        rows = r2;
                    } else {
                        eprintln!("create_test fallback #2 returned 0 rows");
                    }
                }
                Err(e) => {
                    eprintln!("create_test fallback #2 DB error: {}", e);
                }
            }
        }

        // 3) If still empty: try without chapter/range constraints (global random selection)
        if rows.is_empty() {
            eprintln!("create_test fallback #3: selecting from entire questions table (no filters)");
            let query_any = format!("SELECT id, entry_id, q_type, prompt, options, correct_index, correct_answer FROM questions {} {}", order_clause, limit_clause);
            eprintln!("create_test fallback #3 SQL: {}", query_any);
            let mut q_any = sqlx::query(&query_any);
            match q_any.fetch_all(pool).await {
                Ok(r2) => {
                    if !r2.is_empty() {
                        eprintln!("create_test fallback #3 succeeded with {} rows", r2.len());
                        rows = r2;
                    } else {
                        eprintln!("create_test fallback #3 returned 0 rows");
                    }
                }
                Err(e) => {
                    eprintln!("create_test fallback #3 DB error: {}", e);
                }
            }
        }

        // After fallbacks, if still empty return clear error
        if rows.is_empty() {
            eprintln!("create_test: all fallbacks returned 0 rows. SQL primary: {}; binds: {:?}", query_sql, binds);
            return HttpResponse::BadRequest().json(json!({"error":"No questions matched the selection after fallback attempts. Check that entries/questions have correct level/chapter metadata."}));
        }
    }

    // Build question list
    let mut test_questions: Vec<JsonValue> = Vec::new();
    for r in rows {
        let qid: i64 = r.try_get("id").unwrap_or(0);
        let prompt: String = r.try_get("prompt").unwrap_or_default();
        let options_text: String = r.try_get("options").unwrap_or_else(|_| "[]".to_string());
        let options: Vec<String> = serde_json::from_str(&options_text).unwrap_or_default();
        // prefer correct_index, fallback to correct_answer numeric if present
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

    // Insert test into tests table
    let title = format!("Test - {} - {}", level_label, chrono::Utc::now().to_rfc3339());
    let questions_json = match serde_json::to_string(&test_questions) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to serialize test questions: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error":"Failed to serialize test"}));
        }
    };

    let res = match sqlx::query("INSERT INTO tests (title, questions) VALUES (?, ?)")
        .bind(&title)
        .bind(&questions_json)
        .execute(pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to insert test: {}", e);
            return HttpResponse::InternalServerError().json(json!({"error":"Failed to save test"}));
        }
    };

    let test_id = res.last_insert_rowid();
    let redirect = format!("/test/{}", test_id);

    HttpResponse::Created().json(json!({"id": test_id, "redirect": redirect}))
}

/// Get a generated test by id
pub async fn get_test(
    pool: web::Data<SqlitePool>,
    test_id: web::Path<i64>,
) -> impl Responder {
    let pool = pool.get_ref();
    let id = test_id.into_inner();

    // Fetch test row
    let row = match sqlx::query("SELECT id, title, questions, created_at FROM tests WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::NotFound().json(json!({"error": "Test not found"}));
        }
    };

    let questions_text: String = row.try_get("questions").unwrap_or_else(|_| "[]".to_string());
    let questions: JsonValue = match serde_json::from_str(&questions_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse questions JSON for test {}: {}", id, e);
            return HttpResponse::InternalServerError().json(json!({"error":"Failed to parse test data"}));
        }
    };

    let title: String = row.try_get("title").unwrap_or_default();

    HttpResponse::Ok().json(json!({
        "id": id,
        "title": title,
        "questions": questions
    }))
}

/// Debug endpoint: return counts and small samples for troubleshooting.
///
/// Returns JSON including:
/// - total_questions
/// - questions_by_level: array of { level: <id or null>, count: <n> }
/// - questions_by_chapter: array of { chapter: <id or null>, count: <n> } (aggregated from questions.chapter)
/// - sample_questions: small array of sample rows
/// - sample_entries: small array of sample entries
pub async fn debug_stats(pool: web::Data<SqlitePool>) -> impl Responder {
    let pool = pool.get_ref();

    // total questions
    let total_q: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM questions")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // questions by level
    let mut questions_by_level = Vec::new();
    match sqlx::query("SELECT level, COUNT(*) as cnt FROM questions GROUP BY level ORDER BY level ASC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            for r in rows {
                let lvl: Option<i64> = r.try_get("level").ok();
                let cnt: i64 = r.try_get("cnt").unwrap_or(0);
                questions_by_level.push(json!({"level": lvl, "count": cnt}));
            }
        }
        Err(e) => {
            eprintln!("debug_stats: error querying questions by level: {}", e);
        }
    }

    // questions by chapter
    let mut questions_by_chapter = Vec::new();
    match sqlx::query("SELECT chapter, COUNT(*) as cnt FROM questions GROUP BY chapter ORDER BY chapter ASC")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            for r in rows {
                let ch: Option<i64> = r.try_get("chapter").ok();
                let cnt: i64 = r.try_get("cnt").unwrap_or(0);
                questions_by_chapter.push(json!({"chapter": ch, "count": cnt}));
            }
        }
        Err(e) => {
            eprintln!("debug_stats: error querying questions by chapter: {}", e);
        }
    }

    // sample questions
    let mut sample_questions = Vec::new();
    match sqlx::query("SELECT id, entry_id, prompt, level, chapter FROM questions ORDER BY id LIMIT 20")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            for r in rows {
                let id: i64 = r.try_get("id").unwrap_or(0);
                let entry_id: i64 = r.try_get("entry_id").unwrap_or(0);
                let prompt: String = r.try_get("prompt").unwrap_or_default();
                let level: Option<i64> = r.try_get("level").ok();
                let chapter: Option<i64> = r.try_get("chapter").ok();
                sample_questions.push(json!({
                    "id": id,
                    "entry_id": entry_id,
                    "prompt": prompt,
                    "level": level,
                    "chapter": chapter
                }));
            }
        }
        Err(e) => {
            eprintln!("debug_stats: error selecting sample questions: {}", e);
        }
    }

    // sample entries (including chapter)
    let mut sample_entries = Vec::new();
    match sqlx::query("SELECT id, kanji, kana, meaning, chapter FROM entries ORDER BY id LIMIT 20")
        .fetch_all(pool)
        .await
    {
        Ok(rows) => {
            for r in rows {
                let id: i64 = r.try_get("id").unwrap_or(0);
                let kanji: String = r.try_get("kanji").unwrap_or_default();
                let kana: String = r.try_get("kana").unwrap_or_default();
                let meaning: String = r.try_get("meaning").unwrap_or_default();
                let chapter: Option<i64> = r.try_get("chapter").ok();
                sample_entries.push(json!({
                    "id": id,
                    "kanji": kanji,
                    "kana": kana,
                    "meaning": meaning,
                    "chapter": chapter
                }));
            }
        }
        Err(e) => {
            eprintln!("debug_stats: error selecting sample entries: {}", e);
        }
    }

    HttpResponse::Ok().json(json!({
        "total_questions": total_q,
        "questions_by_level": questions_by_level,
        "questions_by_chapter": questions_by_chapter,
        "sample_questions": sample_questions,
        "sample_entries": sample_entries
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/quizzes", web::get().to(list_quizzes))
            .route("/quizzes", web::post().to(create_quiz))
            .route("/quizzes/{id}", web::get().to(get_quiz))
            // test endpoints
            .route("/tests", web::post().to(create_test))
            .route("/tests/{id}", web::get().to(get_test))
            // debug endpoints
            .route("/debug/stats", web::get().to(debug_stats)),
    );
}
