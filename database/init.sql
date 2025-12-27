-- Create database
CREATE DATABASE japanese_vocab;

-- Connect to the database
\c japanese_vocab;

-- Create quizzes table
CREATE TABLE IF NOT EXISTS quizzes (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create questions table
CREATE TABLE IF NOT EXISTS questions (
    id SERIAL PRIMARY KEY,
    quiz_id INTEGER NOT NULL REFERENCES quizzes(id) ON DELETE CASCADE,
    text TEXT NOT NULL,
    options JSONB NOT NULL,
    correct_answer INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_questions_quiz_id ON questions(quiz_id);

-- Insert sample data
INSERT INTO quizzes (title, description) VALUES 
('N5 Basic Greetings', 'Learn basic Japanese greetings'),
('N5 Numbers', 'Japanese numbers from 1 to 10');

INSERT INTO questions (quiz_id, text, options, correct_answer) VALUES
(1, 'What does "こんにちは" (konnichiwa) mean?', '["Hello", "Goodbye", "Thank you", "Sorry"]', 0),
(1, 'What does "ありがとう" (arigatou) mean?', '["Hello", "Goodbye", "Thank you", "Sorry"]', 2),
(1, 'What does "さようなら" (sayounara) mean?', '["Hello", "Goodbye", "Thank you", "Sorry"]', 1),
(2, 'What is the number 1 in Japanese?', '["いち (ichi)", "に (ni)", "さん (san)", "し (shi)"]', 0),
(2, 'What is the number 5 in Japanese?', '["さん (san)", "し (shi)", "ご (go)", "ろく (roku)"]', 2);
