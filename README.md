# Japanese Vocabulary Quiz Application

A full-stack quiz application for learning Japanese vocabulary, built with Node.js frontend, Rust backend, and SQLite database. Designed for localhost single-user deployment with a Quizzi-style interface.

## 🏗️ Architecture

- **Frontend**: Node.js with Express (MVC pattern)
- **Backend**: Rust with Actix-web
- **Database**: SQLite
- **Design**: Quizzi-style quiz interface

## ✨ Features

- 📝 Create custom Japanese vocabulary quizzes
- 🎯 Take quizzes with interactive Quizzi-style UI
- 📊 Real-time scoring (statistics only, not saved to database)
- 🎨 Modern, responsive design
- 🔄 MVC architecture for clean code organization
- 🚀 Fast Rust backend with SQLite database

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v16 or higher)
- **Rust** (latest stable version)
- **Cargo** (comes with Rust)

## 🚀 Installation & Setup

### 1. Clone the Repository

```bash
git clone https://github.com/MikuNakano273/Japanese_vocab.git
cd Japanese_vocab

# Make setup scripts executable
chmod +x setup.sh start.sh
```

### 2. Quick Setup (Recommended)

Use the automated setup script:

```bash
# Run setup script (installs dependencies and builds backend)
./setup.sh
```

### 3. Database Setup

The application uses a SQLite database (`mimikara_n3_questions.db`) that contains Japanese vocabulary questions.

The backend will automatically create and initialize the database file on first run. The database file is included in the repository and contains:
- **1760 vocabulary entries** (kanji, kana, meaning)
- **10232 quiz questions** in various formats (kanji→kana, kana→meaning, etc.)

**Note:** The backend will automatically create required tables and add schema columns on first run if they don't exist.

### 4. Manual Database Inspection

If you want to manually inspect or run SQL against the database, use the `sqlite3` CLI:

```bash
# Connect to the database
sqlite3 mimikara_n3_questions.db

# Examples inside SQLite:
.tables
SELECT COUNT(*) FROM entries;
SELECT COUNT(*) FROM questions;
SELECT * FROM questions LIMIT 5;
```

### 5. Start the Application

#### Quick Start (Both servers at once)

```bash
# Start both frontend and backend servers
./start.sh
```

Access the application at `http://localhost:3000`

#### Manual Start (Separate terminals)

**Terminal 1 - Backend (Rust):**

```bash
cd backend

# Copy environment file (if not done already)
cp .env.example .env

# The default .env uses SQLite:
# DATABASE_URL=sqlite://mimikara_n3_questions.db
# PORT=8081

# Build and run
cargo build --release
cargo run

# The backend will start and connect to SQLite database.
# Default backend URL: http://localhost:8081
```

The backend server will start at `http://localhost:8080`

**Terminal 2 - Frontend (Node.js):**

```bash
cd frontend

# Install dependencies
npm install

# Copy environment file
cp .env.example .env

# Start the server
npm start
```

The frontend server will start at `http://localhost:3000`

## 🎮 Usage

1. **Access the Application**
   - Open your browser and navigate to `http://localhost:3000`

2. **View Quizzes**
   - The home page displays all available quizzes
   - Sample quizzes are loaded automatically from the database

3. **Create a Test from Database** ✨ *New Feature*
   - Click "Create Test" in the navigation bar
   - Select the JLPT level (N5, N4, N3, N2, N1)
   - Choose selection mode:
     - **By Chapter**: Enter chapter numbers (e.g., `1,2,5` or `3-7`)
     - **By Entry Range**: Enter entry ID range (e.g., start: 1, end: 100)
   - Optionally specify the number of questions
   - Click "Create Test" to generate a random test from the database
   - The system will fetch questions from the `mimikara_n3_questions.db` database

4. **Create a Custom Quiz**
   - Click "Create Quiz" in the navigation bar
   - Fill in the quiz title and description
   - Add questions with 4 multiple-choice options
   - Select the correct answer for each question
   - Click "Add Question" to add more questions
   - Submit to create the quiz

5. **Take a Quiz or Test**
   - Click "Start Quiz" on any quiz card or navigate to a generated test
   - Answer questions one by one using the Quizzi-style interface
   - Use "Next" and "Previous" buttons to navigate
   - Submit your answers when complete

6. **View Results**
   - After submission, view your score and percentage
   - Review all questions with correct/incorrect answers highlighted
   - Retake the quiz or return to the quiz list

## 📁 Project Structure

```
Japanese_vocab/
├── frontend/               # Node.js Frontend (MVC)
│   ├── controllers/        # Request handlers
│   ├── models/            # Data models
│   ├── views/             # EJS templates
│   ├── routes/            # Route definitions
│   ├── public/            # Static assets (CSS, JS)
│   ├── app.js             # Main application file
│   └── package.json       # Node.js dependencies
├── backend/               # Rust Backend
│   ├── src/
│   │   ├── db/           # Database connection
│   │   ├── models/       # Data models
│   │   ├── routes/       # API routes
│   │   └── main.rs       # Main application file
│   └── Cargo.toml        # Rust dependencies
└── database/             # Database scripts
    └── init_database_mysql.sh  # MySQL database initialization script
```

## 🔧 Configuration

### Frontend Environment Variables (.env)

```env
BACKEND_URL=http://localhost:8081
PORT=3000
```

### Backend Environment Variables (.env)

```env
DATABASE_URL=sqlite://mimikara_n3_questions.db
PORT=8081
```

## 🛠️ Development

### Frontend Development

```bash
cd frontend
npm run dev
```

### Backend Development

```bash
cd backend
cargo run
```

### Database Management

View tables (using SQLite client):
```bash
sqlite3 mimikara_n3_questions.db
# then in SQLite:
.tables
# or run a query:
SELECT * FROM quizzes;
SELECT * FROM questions;
```

Query data:
```sql
SELECT * FROM quizzes;
SELECT * FROM questions;
```

## 📊 API Endpoints

### Backend REST API

- `GET /api/quizzes` - List all quizzes
- `GET /api/quizzes/:id` - Get a specific quiz
- `POST /api/quizzes` - Create a new quiz

### Example Request (Create Quiz)

```json
POST /api/quizzes
{
  "title": "N5 Vocabulary",
  "description": "Basic Japanese vocabulary",
  "questions": [
    {
      "text": "What does 'ありがとう' mean?",
      "options": ["Hello", "Goodbye", "Thank you", "Sorry"],
      "correct_answer": 2
    }
  ]
}
```

## 🎨 Features Highlight

- **Quizzi-Style Interface**: Modern, card-based quiz interface
- **Real-time Scoring**: Instant feedback without database saves
- **Responsive Design**: Works on desktop and mobile devices
- **Navigation Controls**: Keyboard shortcuts (Arrow keys) support
- **Visual Feedback**: Color-coded correct/incorrect answers
- **Progress Tracking**: See your progress through the quiz

## 📝 Notes

- This application is designed for **localhost single-user** deployment
- Quiz results are **not saved** to the database (statistics only)
- Sample quizzes are included for demonstration purposes
- The application automatically initializes database tables on first run

## 🔐 Security

- CORS is configured for localhost development
- SQL injection prevention through parameterized queries
- Input validation on both frontend and backend

## 📄 License

This project is for educational purposes.

## 👥 Author

MikuNakano273

## 🤝 Contributing

This is a personal project for localhost use. Feel free to fork and modify for your own needs.

---

**Happy Learning Japanese! 🌸**

