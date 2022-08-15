#[cfg(not(debug_assertions))]
use question::Answer;
#[cfg(not(debug_assertions))]
use question::Question;

pub trait QuestionTrait {
    fn yes_no() -> bool;
}
pub struct QuestionWrapped;

#[cfg(not(debug_assertions))]
#[cfg(not(tarpaulin_include))]
impl QuestionTrait for QuestionWrapped {
    fn yes_no() -> bool {
        Question::new("Unsecure HTTP host. Continue? [Y/n]")
            .default(Answer::YES)
            .confirm()
            == Answer::YES
    }
}

#[cfg(debug_assertions)]
#[cfg(not(tarpaulin_include))]
impl QuestionTrait for QuestionWrapped {
    fn yes_no() -> bool {
        true
    }
}
