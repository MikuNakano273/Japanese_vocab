const express = require('express');
const router = express.Router();
const quizController = require('../controllers/quizController');

// Home page
router.get('/', (req, res) => {
    res.redirect('/quiz/list');
});

// Quiz list
router.get('/quiz/list', quizController.listQuizzes);

// Create quiz view
router.get('/quiz/create', quizController.createQuizView);

// Create quiz (POST)
router.post('/quiz/create', quizController.createQuiz);

// Take quiz view
router.get('/quiz/:id', quizController.quizView);

// Submit quiz (calculate score)
router.post('/quiz/:id/submit', quizController.submitQuiz);

module.exports = router;
