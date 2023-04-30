pub mod ndr_parser;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorTypes {
    #[error("Transition vector size not matching number of places: expected {expected:?}")]
    TransitionSizeNotMatching { expected: usize },
    #[error("Cannot assemble graph reason: {reason:?}")]
    CannotAssembleGraph { reason: String },
    #[error("Cannot generate graph as the transition {transition:?} will generate a graph of infinite nodes")]
    PotentialInfiniteGraph { transition: Vec<i32> },
}

#[derive(Debug, Deserialize)]
pub struct Input {
    pub m_names: Vec<String>,
    pub m_init: Vec<Option<i32>>,
    pub transitions: Vec<Vec<i32>>,
}


