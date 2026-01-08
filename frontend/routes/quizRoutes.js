const express = require("express");
const router = express.Router();
const quizController = require("../controllers/quizController");

// Home page
router.get("/", (req, res) => {
  res.redirect("/quiz/list");
});

// Quiz list
router.get("/quiz/list", quizController.listQuizzes);

// Create quiz view
router.get("/quiz/create", quizController.createQuizView);

// Create quiz (POST)
router.post("/quiz/create", quizController.createQuiz);

// Create test view (uses same controller/view as create quiz form which was repurposed)
router.get("/test/create", quizController.createQuizView);

// Create test (POST) - frontend submits to this route to request a generated test
router.post("/test/create", quizController.createTest);

// Test view (render generated test)
router.get("/test/:id", quizController.testView);

// Take quiz view
router.get("/quiz/:id", quizController.quizView);

// Submit quiz (calculate score)
router.post("/quiz/:id/submit", quizController.submitQuiz);

module.exports = router;
