use crate::env::{ArtifactHandle, Value};
use abyss_core::ast::LineInfo;
use colored::*;
use std::fmt;

/// Represents the result of an evaluation in the interpreter.
#[derive(Debug, Clone)]
pub enum EvalResult {
    Data(Value),
    Artifact(ArtifactHandle),
    Revealed(Box<EvalResult>),
    Resume(Option<String>),
    Eject(Option<String>),
}

impl EvalResult {
    pub fn abyss() -> Self {
        EvalResult::Data(Value::Abyss)
    }

    pub fn data(value: Value) -> Self {
        EvalResult::Data(value)
    }

    pub fn artifact(handle: ArtifactHandle) -> Self {
        EvalResult::Artifact(handle)
    }
}

/// Represents possible errors that can occur during evaluation.
#[derive(Debug)]
pub enum EvalError {
    UndefinedVariable(String, Option<LineInfo>),
    InvalidOperation(String, Option<LineInfo>),
    NegativeExponent(Option<LineInfo>),
    TypeError(String, Option<LineInfo>),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(var, _) => write!(f, "Variable {} is not defined!", var),
            EvalError::InvalidOperation(op, _) => write!(f, "Invalid operation: {}", op),
            EvalError::NegativeExponent(_) => {
                write!(f, "PowArcana operation requires a non-negative exponent!")
            }
            EvalError::TypeError(var_type, _) => write!(f, "Type error: {}", var_type),
        }
    }
}

impl std::error::Error for EvalError {}

/// Displays an error message along with the relevant source code and line information, if available.
pub fn display_error_with_source(script: &str, line_info: Option<LineInfo>, error_message: &str) {
    if let Some(info) = line_info {
        let lines: Vec<&str> = script.lines().collect();
        if let Some(source_line) = lines.get(info.line - 1) {
            // Line numbers start from 1, so we subtract 1
            eprintln!(
                "{}",
                format!(
                    "Error at line {}, column {}: {}",
                    info.line, info.column, error_message
                )
                .red()
            );
            eprintln!("  {}", source_line.red());
            eprintln!("  {}{}", " ".repeat(info.column - 1).red(), "^".red());
        } else {
            eprintln!("{}", format!("Error: {}", error_message).red());
        }
    } else {
        eprintln!("{}", format!("Error: {}", error_message).red());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::ArtifactValue;
    use colored::control;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn sample_handle(name: &str) -> ArtifactHandle {
        Rc::new(RefCell::new(ArtifactValue {
            type_name: name.to_string(),
            fields: HashMap::new(),
            field_order: Vec::new(),
        }))
    }

    #[test]
    fn abyss_constructor_returns_abyss_value() {
        match EvalResult::abyss() {
            EvalResult::Data(Value::Abyss) => {}
            other => panic!("expected abyss value, got {:?}", other),
        }
    }

    #[test]
    fn data_constructor_wraps_value() {
        match EvalResult::data(Value::Arcana(42)) {
            EvalResult::Data(Value::Arcana(v)) => assert_eq!(v, 42),
            other => panic!("expected arcana value, got {:?}", other),
        }
    }

    #[test]
    fn artifact_constructor_preserves_handle() {
        let handle = sample_handle("Sigil");
        match EvalResult::artifact(handle.clone()) {
            EvalResult::Artifact(result_handle) => {
                assert!(Rc::ptr_eq(&result_handle, &handle))
            }
            other => panic!("expected artifact handle, got {:?}", other),
        }
    }

    #[test]
    fn display_error_with_valid_line_highlights_source() {
        control::set_override(false);
        let script = "sigil = 1\nhex = sigil + 2";
        display_error_with_source(script, Some(LineInfo::new(2, 5)), "invalid operation");
    }

    #[test]
    fn display_error_without_matching_line_falls_back_to_generic() {
        control::set_override(false);
        display_error_with_source("sigil = 1", Some(LineInfo::new(3, 1)), "out of range");
    }

    #[test]
    fn display_error_without_line_info_still_prints_message() {
        control::set_override(false);
        display_error_with_source("sigil = 1", None, "missing context");
    }
}
