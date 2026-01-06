pub mod evaluate;

pub use evaluate::evaluate;

pub mod nnue;

pub use crate::evaluate::nnue::Nnue;

pub use evaluate::evaluate_neural;