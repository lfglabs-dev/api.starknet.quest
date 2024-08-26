use crate::config::{Quiz, QuizQuestionType};
use crate::models::QuizInsertDocument;
use futures::StreamExt;
use mongodb::bson::{doc, from_document};
use mongodb::Database;
use starknet::core::types::FieldElement;

fn match_vectors(vector1: &Vec<usize>, vector2: &Vec<usize>) -> bool {
    // Check if vectors have the same length
    if vector1.len() != vector2.len() {
        return false;
    }

    // Check if vectors are equal element-wise
    let equal = vector1 == vector2;
    equal
}

// addr is currently unused, this could become the case if we generate
// a deterministic permutation of answers in the future. Seems non necessary for now
#[allow(dead_code)]
pub async fn verify_quiz(
    config: &Database,
    _addr: FieldElement,
    quiz_name: &i64,
    user_answers_list: &Vec<Vec<usize>>,
) -> bool {
    let collection = config.collection::<QuizInsertDocument>("quizzes");
    let pipeline = vec![
        doc! {
            "$match": doc! {
                "id": &quiz_name
            }
        },
        doc! {
            "$lookup": doc! {
                "from": "quiz_questions",
                "let": doc! {
                    "id": "$id"
                },
                "pipeline": [
                    doc! {
                        "$match": doc! {
                            "quiz_id": &quiz_name
                        }
                    },
                    doc! {
                        "$project": doc! {
                            "quiz_id": 0,
                            "_id": 0
                        }
                    }
                ],
                "as": "questions"
            }
        },
        doc! {
            "$project": doc! {
                "_id": 0,
                "id": 0
            }
        },
    ];

    let mut quiz_document = collection.aggregate(pipeline, None).await.unwrap();

    while let Some(result) = quiz_document.next().await {
        match result {
            Ok(document) => {
                let quiz: Quiz = from_document(document).unwrap();
                let mut correct_answers_count = 0;
                for (i, user_answers) in user_answers_list.iter().enumerate() {
                    let question = &quiz.questions[i];
                    let mut user_answers_list = user_answers.clone();
                    let correct_answers: bool = match question.kind {
                        QuizQuestionType::TextChoice => {
                            let mut correct_answers = question.correct_answers.clone().unwrap();
                            correct_answers.sort();
                            user_answers_list.sort();
                            match_vectors(&correct_answers, &user_answers)
                        }
                        QuizQuestionType::ImageChoice => {
                            let mut correct_answers = question.correct_answers.clone().unwrap();
                            correct_answers.sort();
                            user_answers_list.sort();
                            match_vectors(&correct_answers, &user_answers)
                        }
                        QuizQuestionType::Ordering => {
                            let correct_answers = question.correct_answers.clone().unwrap();
                            match_vectors(&correct_answers, &user_answers_list)
                        }
                    };
                    if correct_answers {
                        correct_answers_count += 1;
                    }
                }
                return correct_answers_count == quiz.questions.len();
            }
            Err(_e) => {
                return false;
            }
        }
    }
    true
}
