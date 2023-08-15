use crate::config::{Config, Quiz, QuizQuestionType};
use starknet::core::types::FieldElement;

// addr is currently unused, this could become the case if we generate
// a deterministic permutation of answers in the future. Seems non necessary for now
#[allow(dead_code)]
pub fn verify_quiz(
    config: &Config,
    _addr: FieldElement,
    quiz_name: &str,
    user_answers_list: &Vec<Vec<usize>>,
) -> bool {
    let quiz: &Quiz = match config.quizzes.get(quiz_name) {
        Some(quiz) => quiz,
        None => return false, // Quiz not found
    };

    for (question, user_answers) in quiz.questions.iter().zip(user_answers_list.iter()) {
        match question.kind {
            QuizQuestionType::TextChoice | QuizQuestionType::ImageChoice => {
                if let Some(correct_answers) = &question.correct_answers {
                    // if user_answers does not fit in correct_answers or isn't the same size
                    if user_answers.len() != correct_answers.len()
                        || !user_answers
                            .iter()
                            .all(|&item| correct_answers.contains(&item))
                    {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            QuizQuestionType::Ordering => {
                if let Some(correct_order) = &question.correct_order {
                    if correct_order != user_answers {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
    true
}
