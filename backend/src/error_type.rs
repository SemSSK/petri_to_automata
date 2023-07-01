use thiserror::Error;

/// description of the possivle errors
#[derive(Error, Debug)]
pub enum ErrorTypes {
    #[error("Transition vector size not matching number of places: expected {expected:?}")]
    TransitionSizeNotMatching { expected: usize },
    #[error("Cannot assemble graph reason: {reason:?}")]
    CannotAssembleGraph { reason: String },
    #[error("Cannot generate graph as the transition {transition:?} will generate a graph of infinite nodes")]
    PotentialInfiniteGraph { transition: Vec<i32> },
    #[error("{reason:?}")]
    BadTransition { reason: String },
    #[error("Repeating place")]
    BadPlace,
}
