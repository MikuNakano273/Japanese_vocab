const axios = require("axios");

// Use backend port 8081 (SQLite-based backend) as default
const BACKEND_URL = process.env.BACKEND_URL || "http://localhost:8081";

// List all quizzes
exports.listQuizzes = async (req, res) => {
  try {
    const response = await axios.get(`${BACKEND_URL}/api/quizzes`);
    res.render("quizList", {
      title: "Japanese Vocabulary Quizzes",
      quizzes: response.data,
    });
  } catch (error) {
    console.error("Error fetching quizzes:", error.message);
    res.render("quizList", {
      title: "Japanese Vocabulary Quizzes",
      quizzes: [],
    });
  }
};

// Show create quiz view
exports.createQuizView = (req, res) => {
  res.render("createQuiz", {
    title: "Create New Quiz",
  });
};

// Create new quiz
exports.createQuiz = async (req, res) => {
  try {
    const response = await axios.post(`${BACKEND_URL}/api/quizzes`, req.body);
    res.redirect("/quiz/list");
  } catch (error) {
    console.error("Error creating quiz:", error.message);
    res.render("createQuiz", {
      title: "Create New Quiz",
      error: "Failed to create quiz. Please try again.",
    });
  }
};

// Create new test (API-backed)
// Expects a JSON body with:
// {
//   level: "n4",
//   mode: "chapter" | "range",
//   chapters: [1,2,3] or "1,3-5",        // when mode === "chapter"
//   range: { start: 1, end: 100 } or "33-68", // when mode === "range"
//   numQuestions: 20
// }
// Responds with JSON: { redirect: "/test/<id>", id: <id> } on success
exports.createTest = async (req, res) => {
  try {
    // Helper: parse chapters input supporting "1,2,5-8" and arrays
    function parseChapters(input) {
      if (!input) return [];
      if (Array.isArray(input)) {
        return input.map((n) => Number(n)).filter((n) => !Number.isNaN(n));
      }
      if (typeof input === "string") {
        const parts = input
          .split(",")
          .map((s) => s.trim())
          .filter(Boolean);
        const set = new Set();
        for (const p of parts) {
          // simple number
          if (/^\d+$/.test(p)) {
            set.add(Number(p));
            continue;
          }
          // range like 3-7 or 7 - 3
          const m = p.match(/^(\d+)\s*-\s*(\d+)$/);
          if (m) {
            let a = Number(m[1]),
              b = Number(m[2]);
            if (Number.isNaN(a) || Number.isNaN(b)) continue;
            const start = Math.min(a, b),
              end = Math.max(a, b);
            for (let i = start; i <= end; i++) set.add(i);
          }
        }
        return Array.from(set).sort((a, b) => a - b);
      }
      // unknown type
      return [];
    }

    // Helper: parse range input supporting object or string "start-end"
    function parseRange(r) {
      if (!r) return null;
      if (typeof r === "object" && r.start != null && r.end != null) {
        const s = Number(r.start),
          e = Number(r.end);
        if (Number.isNaN(s) || Number.isNaN(e)) return null;
        return { start: Math.min(s, e), end: Math.max(s, e) };
      }
      if (typeof r === "string") {
        const m = r.match(/^\s*(\d+)\s*-\s*(\d+)\s*$/);
        if (m) {
          const s = Number(m[1]),
            e = Number(m[2]);
          if (Number.isNaN(s) || Number.isNaN(e)) return null;
          return { start: Math.min(s, e), end: Math.max(s, e) };
        }
      }
      return null;
    }

    // Build and normalize payload
    const level = req.body.level;
    const mode = req.body.mode;
    const numQuestions = req.body.numQuestions
      ? Number(req.body.numQuestions)
      : null;

    let chapters = parseChapters(req.body.chapters);
    let range = parseRange(req.body.range);

    // Basic validation
    if (mode === "chapter") {
      if ((!chapters || chapters.length === 0) && !numQuestions) {
        return res
          .status(400)
          .json({
            error:
              "Must provide chapter numbers or numQuestions when mode='chapter'",
          });
      }
    } else if (mode === "range") {
      if (!range) {
        return res
          .status(400)
          .json({
            error:
              "Range must be provided as {start,end} or 'start-end' string when mode='range'",
          });
      }
    }

    const payload = {
      level: level,
      mode: mode,
      chapters: chapters.length ? chapters : null,
      range: range,
      numQuestions: numQuestions,
    };

    // Forward the request to backend test-generation endpoint.
    const response = await axios.post(`${BACKEND_URL}/api/tests`, payload);
    const data = response.data || {};

    // If backend provides a redirect URL, return it to the frontend (AJAX caller).
    if (data.redirect) {
      return res.json({ redirect: data.redirect, id: data.id || null });
    }

    // Otherwise, construct a redirect to the test view.
    if (data.id) {
      return res.json({ redirect: `/test/${data.id}`, id: data.id });
    }

    // If backend returned unexpected payload, still respond with success and allow frontend to navigate.
    return res.status(201).json({ redirect: "/", id: null });
  } catch (error) {
    // Better error reporting: include backend response data if available
    console.error(
      "Error creating test:",
      error && error.message ? error.message : error,
    );
    if (error.response && error.response.data) {
      console.error("Backend response:", error.response.data);
    }
    // Return JSON error to the AJAX caller with helpful message
    res
      .status(500)
      .json({
        error:
          "Failed to create test. Please check your selection and try again.",
      });
  }
};

// Show quiz taking view
exports.quizView = async (req, res) => {
  try {
    const response = await axios.get(
      `${BACKEND_URL}/api/quizzes/${req.params.id}`,
    );
    res.render("quiz", {
      title: response.data.title,
      quiz: response.data,
    });
  } catch (error) {
    console.error("Error fetching quiz:", error.message);
    res.status(404).render("404", { title: "Quiz Not Found" });
  }
};

// Show test view (renders interactive test UI)
// Expects backend to expose GET /api/tests/:id returning a test object compatible with the frontend view.
exports.testView = async (req, res) => {
  try {
    const response = await axios.get(
      `${BACKEND_URL}/api/tests/${req.params.id}`,
    );
    const test = response.data;
    // Render a 'test' view (you should add frontend/views/test.ejs) that can present
    // questions and immediate-feedback behavior. For now we attempt to render 'quiz'
    // if 'test' template is not present; better to create a dedicated 'test.ejs'.
    // Pass BACKEND_URL so client-side JS can call backend if needed.
    const viewName = "test"; // create 'test.ejs' to get the intended UI
    res.render(viewName, {
      title: test.title || "Test",
      test: test,
      BACKEND_URL: process.env.BACKEND_URL || BACKEND_URL,
    });
  } catch (error) {
    console.error(
      "Error fetching test:",
      error && error.message ? error.message : error,
    );
    res.status(404).render("404", { title: "Test Not Found" });
  }
};

// Submit quiz and calculate score
exports.submitQuiz = async (req, res) => {
  try {
    const quizId = req.params.id;
    const userAnswers = req.body.answers;

    // Fetch quiz to validate answers
    const quizResponse = await axios.get(
      `${BACKEND_URL}/api/quizzes/${quizId}`,
    );
    const quiz = quizResponse.data;

    // Calculate score
    let score = 0;
    let totalQuestions = quiz.questions.length;

    quiz.questions.forEach((question, index) => {
      const userAnswer = parseInt(userAnswers[index]);
      if (userAnswer === question.correct_answer) {
        score++;
      }
    });

    const percentage = ((score / totalQuestions) * 100).toFixed(2);

    // Return results (not saved to database)
    res.render("quizResult", {
      title: "Quiz Results",
      quiz: quiz,
      score: score,
      total: totalQuestions,
      percentage: percentage,
      userAnswers: userAnswers,
    });
  } catch (error) {
    console.error("Error submitting quiz:", error.message);
    res.status(500).send("Error processing quiz submission");
  }
};
