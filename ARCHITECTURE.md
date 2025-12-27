# Project Summary - Japanese Vocabulary Quiz Application

## Overview
This is a complete full-stack web application for creating and taking Japanese vocabulary quizzes, designed for localhost single-user deployment.

## Architecture

### Technology Stack
- **Frontend**: Node.js + Express.js (MVC pattern)
- **Backend**: Rust + Actix-web
- **Database**: PostgreSQL
- **Template Engine**: EJS
- **Styling**: Custom CSS (Quizzi-style design)

### MVC Pattern Implementation

#### Frontend MVC Structure
```
frontend/
├── models/          # (Currently minimal - data comes from backend)
├── views/           # EJS templates for UI
│   ├── quizList.ejs       # List all quizzes
│   ├── createQuiz.ejs     # Create new quiz
│   ├── quiz.ejs           # Take quiz (Quizzi-style)
│   └── quizResult.ejs     # View results
├── controllers/     # Request handlers
│   └── quizController.js
└── routes/          # URL routing
    └── quizRoutes.js
```

#### Backend Structure (Rust)
```
backend/src/
├── models/          # Data structures
├── routes/          # API endpoints
├── db/              # Database connection
└── main.rs          # Application entry point
```

## Key Features

### 1. Quiz Creation View
- Add title and description
- Add multiple questions dynamically
- Each question has 4 options (A, B, C, D)
- Select correct answer for each question
- Form validation

### 2. Quiz Taking View (Quizzi-style)
- Card-based interface
- One question at a time
- Progress indicator
- Navigation buttons (Next/Previous)
- Keyboard shortcuts (Arrow keys)
- Answer validation before submission

### 3. Results Display
- Percentage score with visual circle
- Score breakdown (correct/total)
- Review all questions
- Color-coded correct/incorrect answers
- Option to retake quiz

### 4. Scoring System
- Real-time calculation
- Not saved to database (as requested)
- Instant feedback on submission

## Database Schema

### Tables

#### `quizzes`
```sql
- id (SERIAL PRIMARY KEY)
- title (VARCHAR)
- description (TEXT)
- created_at (TIMESTAMP)
```

#### `questions`
```sql
- id (SERIAL PRIMARY KEY)
- quiz_id (FOREIGN KEY)
- text (TEXT)
- options (JSONB) - Array of 4 options
- correct_answer (INTEGER) - Index 0-3
- created_at (TIMESTAMP)
```

## API Endpoints

### Backend REST API (http://localhost:8080)

1. **GET /api/quizzes**
   - List all available quizzes
   - Returns: Array of quiz objects with questions

2. **GET /api/quizzes/:id**
   - Get specific quiz by ID
   - Returns: Quiz object with all questions

3. **POST /api/quizzes**
   - Create new quiz
   - Body: { title, description, questions[] }
   - Returns: Created quiz ID

## Frontend Routes

### User-facing URLs (http://localhost:3000)

1. **GET /**
   - Redirects to quiz list

2. **GET /quiz/list**
   - Display all quizzes

3. **GET /quiz/create**
   - Show quiz creation form

4. **POST /quiz/create**
   - Submit new quiz

5. **GET /quiz/:id**
   - Take specific quiz

6. **POST /quiz/:id/submit**
   - Submit quiz answers and get results

## Design Philosophy

### Quizzi-Style Interface
- Modern card-based design
- Clean typography
- Smooth transitions
- Visual feedback
- Responsive layout

### Color Scheme
- Primary: Indigo (#6366f1)
- Success: Green (#10b981)
- Error: Red (#ef4444)
- Background: Light gray (#f3f4f6)

## Setup & Installation

### Prerequisites
- Node.js 16+
- Rust (latest stable)
- PostgreSQL 12+

### Quick Start
```bash
# 1. Setup environment and dependencies
./setup.sh

# 2. Initialize database
psql -U postgres -f database/init.sql

# 3. Start application (starts both servers)
./start.sh
```

### Manual Start
```bash
# Terminal 1 - Backend
cd backend
cargo run

# Terminal 2 - Frontend
cd frontend
npm start
```

Access: http://localhost:3000

## Project Structure

```
Japanese_vocab/
├── frontend/              # Node.js MVC Application
│   ├── controllers/       # Business logic
│   ├── views/            # EJS templates
│   ├── routes/           # URL routing
│   ├── public/           # Static assets
│   │   ├── css/         # Stylesheets
│   │   └── js/          # Client-side JavaScript
│   └── app.js           # Express server
│
├── backend/              # Rust API Server
│   └── src/
│       ├── db/          # Database connection
│       ├── models/      # Data structures
│       ├── routes/      # API routes
│       └── main.rs      # Server entry point
│
├── database/            # Database scripts
│   └── init.sql        # Schema + sample data
│
├── docker-compose.yml  # PostgreSQL container
├── setup.sh           # Setup script
└── start.sh           # Start script
```

## Technical Highlights

### Frontend
- Express.js for server-side rendering
- EJS for templating
- Axios for API calls
- Custom JavaScript for quiz interactivity
- Responsive CSS design

### Backend
- Actix-web (high-performance async framework)
- PostgreSQL with tokio-postgres
- JSON support via postgres-types
- CORS enabled for development
- Async/await pattern

### Database
- PostgreSQL with JSONB for flexible option storage
- Foreign key constraints
- Automatic timestamps
- Sample data included

## Security Features

- Parameterized SQL queries (prevents SQL injection)
- CORS configured for localhost
- Input validation on both frontend and backend
- No sensitive data exposure

## Performance Considerations

- Rust backend for high performance
- Connection pooling ready (Arc<Mutex<Client>>)
- Minimal frontend JavaScript
- CSS-only animations
- Single-page quiz navigation (no reloads)

## Future Enhancements (Not Implemented)

Possible additions:
- User authentication
- Quiz categories/tags
- Timed quizzes
- Multiple quiz types (fill-in-blank, matching)
- Export/import quizzes
- Statistics dashboard
- Audio pronunciation for Japanese words

## Notes

- Application is designed for **single-user localhost** deployment
- Quiz results are **not persisted** (scoring is calculated on-demand)
- Sample Japanese vocabulary quizzes included
- Database tables auto-initialize on first backend run

## Troubleshooting

### Backend won't start
- Check PostgreSQL is running
- Verify DATABASE_URL in backend/.env
- Run: `cargo clean && cargo build`

### Frontend won't start
- Run: `npm install` in frontend/
- Check port 3000 is available
- Verify BACKEND_URL in frontend/.env

### Database connection fails
- Ensure PostgreSQL service is running
- Check credentials match .env file
- Run init.sql to create database

## License
Educational purposes - Free to use and modify

## Author
MikuNakano273

---
Built with ❤️ for Japanese language learners 🌸
