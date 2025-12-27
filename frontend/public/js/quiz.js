let currentQuestion = 0;
const questionCards = document.querySelectorAll('.question-card');
const totalQuestions = questionCards.length;

function showQuestion(index) {
    questionCards.forEach((card, i) => {
        if (i === index) {
            card.classList.add('active');
        } else {
            card.classList.remove('active');
        }
    });
    
    currentQuestion = index;
    updateProgress();
}

function updateProgress() {
    document.getElementById('currentQuestion').textContent = currentQuestion + 1;
}

function nextQuestion() {
    if (currentQuestion < totalQuestions - 1) {
        // Check if current question is answered
        const currentCard = questionCards[currentQuestion];
        const selectedOption = currentCard.querySelector('input[type="radio"]:checked');
        
        if (!selectedOption) {
            alert('Please select an answer before proceeding.');
            return;
        }
        
        showQuestion(currentQuestion + 1);
    }
}

function previousQuestion() {
    if (currentQuestion > 0) {
        showQuestion(currentQuestion - 1);
    }
}

// Add keyboard navigation
document.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowRight') {
        nextQuestion();
    } else if (e.key === 'ArrowLeft') {
        previousQuestion();
    }
});

// Form submission validation
document.getElementById('quizForm').addEventListener('submit', (e) => {
    const unanswered = [];
    
    questionCards.forEach((card, index) => {
        const selectedOption = card.querySelector('input[type="radio"]:checked');
        if (!selectedOption) {
            unanswered.push(index + 1);
        }
    });
    
    if (unanswered.length > 0) {
        e.preventDefault();
        alert(`Please answer all questions. Unanswered: Question ${unanswered.join(', ')}`);
        showQuestion(unanswered[0] - 1);
    }
});

// Initialize
updateProgress();
