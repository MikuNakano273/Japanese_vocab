use actix_web::{web, HttpResponse, Responder};
use tokio_postgres::Client;
use tokio_postgres::types::Json;
use crate::models::{CreateQuizRequest, Quiz, Question};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn list_quizzes(client: web::Data<Arc<Mutex<Client>>>) -> impl Responder {
    let client = client.lock().await;
    
    let rows = match client
        .query("SELECT id, title, description FROM quizzes ORDER BY created_at DESC", &[])
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch quizzes"
            }));
        }
    };
    
    let mut quizzes = Vec::new();
    for row in rows {
        let quiz_id: i32 = row.get(0);
        
        // Get questions count for each quiz
        let questions_rows = client
            .query(
                "SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = $1",
                &[&quiz_id],
            )
            .await
            .unwrap_or_default();
        
        let questions: Vec<Question> = questions_rows
            .iter()
            .map(|row| {
                let options_json: Json<Vec<String>> = row.get(2);
                
                Question {
                    id: Some(row.get(0)),
                    text: row.get(1),
                    options: options_json.0,
                    correct_answer: row.get(3),
                }
            })
            .collect();
        
        quizzes.push(Quiz {
            id: quiz_id,
            title: row.get(1),
            description: row.get(2),
            questions,
        });
    }
    
    HttpResponse::Ok().json(quizzes)
}

pub async fn get_quiz(
    client: web::Data<Arc<Mutex<Client>>>,
    quiz_id: web::Path<i32>,
) -> impl Responder {
    let client = client.lock().await;
    let id = quiz_id.into_inner();
    
    let quiz_row = match client
        .query_one("SELECT id, title, description FROM quizzes WHERE id = $1", &[&id])
        .await
    {
        Ok(row) => row,
        Err(_) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Quiz not found"
            }));
        }
    };
    
    let questions_rows = match client
        .query(
            "SELECT id, text, options, correct_answer FROM questions WHERE quiz_id = $1",
            &[&id],
        )
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch questions"
            }));
        }
    };
    
    let questions: Vec<Question> = questions_rows
        .iter()
        .map(|row| {
            let options_json: Json<Vec<String>> = row.get(2);
            
            Question {
                id: Some(row.get(0)),
                text: row.get(1),
                options: options_json.0,
                correct_answer: row.get(3),
            }
        })
        .collect();
    
    let quiz = Quiz {
        id: quiz_row.get(0),
        title: quiz_row.get(1),
        description: quiz_row.get(2),
        questions,
    };
    
    HttpResponse::Ok().json(quiz)
}

pub async fn create_quiz(
    client: web::Data<Arc<Mutex<Client>>>,
    quiz_data: web::Json<CreateQuizRequest>,
) -> impl Responder {
    let client = client.lock().await;
    
    // Insert quiz
    let quiz_row = match client
        .query_one(
            "INSERT INTO quizzes (title, description) VALUES ($1, $2) RETURNING id",
            &[&quiz_data.title, &quiz_data.description],
        )
        .await
    {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create quiz"
            }));
        }
    };
    
    let quiz_id: i32 = quiz_row.get(0);
    
    // Insert questions
    for question in &quiz_data.questions {
        let options_json = Json(&question.options);
        
        if let Err(e) = client
            .execute(
                "INSERT INTO questions (quiz_id, text, options, correct_answer) VALUES ($1, $2, $3, $4)",
                &[&quiz_id, &question.text, &options_json, &question.correct_answer],
            )
            .await
        {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create questions"
            }));
        }
    }
    
    HttpResponse::Created().json(serde_json::json!({
        "id": quiz_id,
        "message": "Quiz created successfully"
    }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/quizzes", web::get().to(list_quizzes))
            .route("/quizzes", web::post().to(create_quiz))
            .route("/quizzes/{id}", web::get().to(get_quiz)),
    );
}
