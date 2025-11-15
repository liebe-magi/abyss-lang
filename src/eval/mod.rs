mod collections;
mod expressions;
mod result;
mod statements;
mod values;

pub use result::{EvalError, EvalResult, display_error_with_source};
pub use statements::evaluate;
