use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quiz {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<Question>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Question {
    pub id: Option<i32>,
    pub text: String,
    pub options: Vec<String>,
    pub correct_answer: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuizRequest {
    pub title: String,
    pub description: Option<String>,
    pub questions: Vec<QuestionInput>,
}

#[derive(Debug, Deserialize)]
pub struct QuestionInput {
    pub text: String,
    pub options: Vec<String>,
    pub correct_answer: i32,
}

// Test-related types and payloads removed because they are not used by current code.
// If you need to reintroduce test generation payloads or models later, re-add
// appropriate structs here.
