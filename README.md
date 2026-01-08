# Japanese Vocabulary Quiz Application

A full-stack quiz application for learning Japanese vocabulary, built with Node.js frontend, Rust backend, and SQLite (file-based) database. Designed for localhost single-user deployment with a Quizzi-style interface. The application uses a local SQLite file `mimikara_n3_question.db` by default.

## 🏗️ Architecture

- **Frontend**: Node.js with Express (MVC pattern)
- **Backend**: Rust with Actix-web
- **Database**: SQLite (file: `mimikara_n3_question.db`)
- **Design**: Quizzi-style quiz interface

## ✨ Features

- 📝 Create custom Japanese vocabulary quizzes
- 🎯 Take quizzes with interactive Quizzi-style UI
- 📊 Real-time scoring (statistics only, not saved to database)
- 🎨 Modern, responsive design
- 🔄 MVC architecture for clean code organization
- 🚀 Fast Rust backend with SQLite (file-based database)

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v16 or higher)
- **Rust** (latest stable version)
- **SQLite** (file-based database; no separate server required)
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

Database initialization is automatic on first backend run. The backend will create the SQLite file `mimikara_n3_question.db` (if it doesn't exist) and initialize required tables.

If you prefer to inspect the database manually, you can use the sqlite3 CLI:

```bash
# Inspect the SQLite database file (after running the backend at least once)
sqlite3 mimikara_n3_question.db
# In sqlite3 shell:
# .tables
# SELECT * FROM quizzes;
```

### 3. Manual Database Setup

No separate database server is required. SQLite is file-based and the backend will create and initialize the database file automatically when started.

If you want to manually inspect or run SQL against the database file, use the `sqlite3` CLI:

```bash
# Open the database file
sqlite3 mimikara_n3_question.db

# Examples inside sqlite3:
.tables
SELECT * FROM quizzes;
SELECT * FROM questions;
```

Or manually:

```bash
# No separate DB server required — the backend will create and initialize the SQLite file `mimikara_n3_question.db`.
# If you want to inspect the database manually, use the sqlite3 CLI after running the backend at least once:
sqlite3 mimikara_n3_question.db
# In the sqlite3 shell:
# .tables
# SELECT * FROM quizzes;
```

### 4. Start the Application

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

# Edit .env if needed to change the SQLite database location
# Example:
# DATABASE_URL=sqlite://mimikara_n3_question.db
# PORT=8081

# Build and run
cargo build --release
cargo run

# The backend will start and (if needed) create `mimikara_n3_question.db` and initialize tables.
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

3. **Create a Quiz**
   - Click "Create Quiz" in the navigation bar
   - Fill in the quiz title and description
   - Add questions with 4 multiple-choice options
   - Select the correct answer for each question
   - Click "Add Question" to add more questions
   - Submit to create the quiz

4. **Take a Quiz**
   - Click "Start Quiz" on any quiz card
   - Answer questions one by one using the Quizzi-style interface
   - Use "Next" and "Previous" buttons to navigate
   - Submit your answers when complete

5. **View Results**
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
    └── (no separate SQL init file — database is initialized automatically by the backend into `mimikara_n3_question.db`)
```

## 🔧 Configuration

### Frontend Environment Variables (.env)

```env
BACKEND_URL=http://localhost:8081
PORT=3000
```

### Backend Environment Variables (.env)

```env
DATABASE_URL=sqlite://mimikara_n3_question.db
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

View tables (using sqlite3 after the backend has initialized the DB file):
```bash
sqlite3 mimikara_n3_question.db
# then in sqlite3:
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

