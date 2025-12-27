const axios = require('axios');

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

// List all quizzes
exports.listQuizzes = async (req, res) => {
    try {
        const response = await axios.get(`${BACKEND_URL}/api/quizzes`);
        res.render('quizList', { 
            title: 'Japanese Vocabulary Quizzes',
            quizzes: response.data 
        });
    } catch (error) {
        console.error('Error fetching quizzes:', error.message);
        res.render('quizList', { 
            title: 'Japanese Vocabulary Quizzes',
            quizzes: [] 
        });
    }
};

// Show create quiz view
exports.createQuizView = (req, res) => {
    res.render('createQuiz', { 
        title: 'Create New Quiz' 
    });
};

// Create new quiz
exports.createQuiz = async (req, res) => {
    try {
        const response = await axios.post(`${BACKEND_URL}/api/quizzes`, req.body);
        res.redirect('/quiz/list');
    } catch (error) {
        console.error('Error creating quiz:', error.message);
        res.render('createQuiz', { 
            title: 'Create New Quiz',
            error: 'Failed to create quiz. Please try again.' 
        });
    }
};

// Show quiz taking view
exports.quizView = async (req, res) => {
    try {
        const response = await axios.get(`${BACKEND_URL}/api/quizzes/${req.params.id}`);
        res.render('quiz', { 
            title: response.data.title,
            quiz: response.data 
        });
    } catch (error) {
        console.error('Error fetching quiz:', error.message);
        res.status(404).render('404', { title: 'Quiz Not Found' });
    }
};

// Submit quiz and calculate score
exports.submitQuiz = async (req, res) => {
    try {
        const quizId = req.params.id;
        const userAnswers = req.body.answers;
        
        // Fetch quiz to validate answers
        const quizResponse = await axios.get(`${BACKEND_URL}/api/quizzes/${quizId}`);
        const quiz = quizResponse.data;
        
        // Calculate score
        let score = 0;
        let totalQuestions = quiz.questions.length;
        
        quiz.questions.forEach((question, index) => {
            const userAnswer = userAnswers[index];
            if (userAnswer === question.correct_answer) {
                score++;
            }
        });
        
        const percentage = ((score / totalQuestions) * 100).toFixed(2);
        
        // Return results (not saved to database)
        res.render('quizResult', {
            title: 'Quiz Results',
            quiz: quiz,
            score: score,
            total: totalQuestions,
            percentage: percentage,
            userAnswers: userAnswers
        });
    } catch (error) {
        console.error('Error submitting quiz:', error.message);
        res.status(500).send('Error processing quiz submission');
    }
};
