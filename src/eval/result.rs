use crate::ast::LineInfo;
use crate::env::{ArtifactHandle, Value};
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
