# Japanese Vocabulary Quiz Application

A full-stack quiz application for learning Japanese vocabulary, built with Node.js frontend, Rust backend, and PostgreSQL database. Designed for localhost single-user deployment with a Quizzi-style interface.

## 🏗️ Architecture

- **Frontend**: Node.js with Express (MVC pattern)
- **Backend**: Rust with Actix-web
- **Database**: PostgreSQL
- **Design**: Quizzi-style quiz interface

## ✨ Features

- 📝 Create custom Japanese vocabulary quizzes
- 🎯 Take quizzes with interactive Quizzi-style UI
- 📊 Real-time scoring (statistics only, not saved to database)
- 🎨 Modern, responsive design
- 🔄 MVC architecture for clean code organization
- 🚀 Fast Rust backend with PostgreSQL

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v16 or higher)
- **Rust** (latest stable version)
- **PostgreSQL** (v12 or higher)
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

Then initialize the database:

```bash
# Create database and tables
psql -U postgres -f database/init.sql
```

### 3. Manual Database Setup

Start PostgreSQL and create the database:

```bash
# Start PostgreSQL service (depends on your OS)
# For Ubuntu/Debian:
sudo service postgresql start

# For macOS (using Homebrew):
brew services start postgresql

# Create database and tables
psql -U postgres -f database/init.sql
```

Or manually:

```bash
psql -U postgres
CREATE DATABASE japanese_vocab;
\q
psql -U postgres -d japanese_vocab -f database/init.sql
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

# Edit .env if needed to match your PostgreSQL credentials
# DATABASE_URL=host=localhost user=postgres password=postgres dbname=japanese_vocab

# Build and run
cargo build --release
cargo run
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
    └── init.sql          # Database initialization
```

## 🔧 Configuration

### Frontend Environment Variables (.env)

```env
BACKEND_URL=http://localhost:8080
PORT=3000
```

### Backend Environment Variables (.env)

```env
DATABASE_URL=host=localhost user=postgres password=postgres dbname=japanese_vocab
PORT=8080
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

View tables:
```bash
psql -U postgres -d japanese_vocab
\dt
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

