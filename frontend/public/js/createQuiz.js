let questionCount = 1;

function addQuestion() {
    const container = document.getElementById('questionsContainer');
    questionCount++;
    
    const questionBlock = document.createElement('div');
    questionBlock.className = 'question-block';
    questionBlock.setAttribute('data-question-index', questionCount - 1);
    
    questionBlock.innerHTML = `
        <h3>Question ${questionCount}</h3>
        <div class="form-group">
            <label>Question Text:</label>
            <input type="text" name="questions[${questionCount - 1}][text]" required placeholder="Enter your question">
        </div>
        <div class="form-group">
            <label>Option A:</label>
            <input type="text" name="questions[${questionCount - 1}][options][0]" required placeholder="Option A">
        </div>
        <div class="form-group">
            <label>Option B:</label>
            <input type="text" name="questions[${questionCount - 1}][options][1]" required placeholder="Option B">
        </div>
        <div class="form-group">
            <label>Option C:</label>
            <input type="text" name="questions[${questionCount - 1}][options][2]" required placeholder="Option C">
        </div>
        <div class="form-group">
            <label>Option D:</label>
            <input type="text" name="questions[${questionCount - 1}][options][3]" required placeholder="Option D">
        </div>
        <div class="form-group">
            <label>Correct Answer:</label>
            <select name="questions[${questionCount - 1}][correct_answer]" required>
                <option value="0">A</option>
                <option value="1">B</option>
                <option value="2">C</option>
                <option value="3">D</option>
            </select>
        </div>
        <button type="button" class="btn btn-secondary" onclick="removeQuestion(this)">Remove Question</button>
    `;
    
    container.appendChild(questionBlock);
}

function removeQuestion(button) {
    const questionBlock = button.closest('.question-block');
    questionBlock.remove();
    questionCount--;
    updateQuestionNumbers();
}

function updateQuestionNumbers() {
    const questions = document.querySelectorAll('.question-block');
    questions.forEach((block, index) => {
        const h3 = block.querySelector('h3');
        h3.textContent = `Question ${index + 1}`;
    });
}

document.getElementById('createQuizForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    
    const formData = new FormData(e.target);
    const data = {
        title: formData.get('title'),
        description: formData.get('description'),
        questions: []
    };
    
    // Parse questions
    const questionBlocks = document.querySelectorAll('.question-block');
    questionBlocks.forEach((block, index) => {
        const question = {
            text: formData.get(`questions[${index}][text]`),
            options: [
                formData.get(`questions[${index}][options][0]`),
                formData.get(`questions[${index}][options][1]`),
                formData.get(`questions[${index}][options][2]`),
                formData.get(`questions[${index}][options][3]`)
            ],
            correct_answer: parseInt(formData.get(`questions[${index}][correct_answer]`))
        };
        data.questions.push(question);
    });
    
    try {
        const response = await fetch('/quiz/create', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            window.location.href = '/quiz/list';
        } else {
            alert('Failed to create quiz. Please try again.');
        }
    } catch (error) {
        console.error('Error:', error);
        alert('An error occurred. Please try again.');
    }
});
