use crate::ast::{AST, LineInfo, Type};
use crate::eval::{EvalError, EvalResult};
use std::collections::HashMap;

pub type BuiltinFunc =
    fn(&mut Environment, Vec<CallArg>, Option<LineInfo>) -> Result<EvalResult, EvalError>;

#[derive(Debug)]
pub struct CallArg {
    pub value: EvalResult,
    pub var_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Engraved(EngravedFunction),
    Builtin(BuiltinFunction),
}

#[derive(Debug, Clone)]
pub struct EngravedFunction {
    pub name: String,
    pub params: Vec<AST>,
    pub return_type: Type,
    pub body: Box<AST>,
    pub line_info: Option<LineInfo>,
}

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub func: BuiltinFunc,
}

/// Stores information about a variable, including its value, type, and mutability.
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub value: Value,
    pub var_type: Type,
    pub is_morph: bool,
    pub line_info: Option<LineInfo>,
}

/// Manages variable and function scopes in the execution environment, including
/// both the global scope and any nested local scopes.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, VarInfo>>, // Variable scopes
    function_scopes: Vec<HashMap<String, Callable>>, // Function scopes
}

impl Environment {
    /// Creates a new environment with an initial global scope.
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
            function_scopes: vec![HashMap::new()],
        }
    }

    /// Pushes a new scope onto the stack, creating a new local environment for variables and functions.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.function_scopes.push(HashMap::new());
    }

    /// Pops the most recent scope off the stack, discarding the current local environment.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.function_scopes.pop();
    }

    /// Sets a variable in the current scope, specifying its name, value, type, and whether it's mutable.
    pub fn set_var(
        &mut self,
        name: String,
        value: Value,
        var_type: Type,
        is_morph: bool,
        line_info: Option<LineInfo>,
    ) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(
                name,
                VarInfo {
                    value,
                    var_type,
                    is_morph,
                    line_info,
                },
            );
        }
    }

    /// Retrieves a variable from the environment by searching the scopes from the most recent to the global scope.
    pub fn get_var(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(var_info) = scope.get(name) {
                return Some(var_info);
            }
        }
        None
    }

    pub fn get_var_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                return Some(var_info);
            }
        }
        None
    }

    /// Updates an existing variable's value in the environment if it is mutable and the types match.
    /// Returns an error if the variable is immutable, the types do not match, or the variable is not found.
    pub fn update_var(
        &mut self,
        name: &str,
        value: Value,
        var_type: Type,
        line_info: Option<LineInfo>,
    ) -> Result<(), EvalError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                if !var_info.is_morph {
                    return Err(EvalError::InvalidOperation(
                        format!("Cannot reassign to immutable variable {}", name),
                        line_info,
                    ));
                }

                if var_info.var_type != var_type
                    && var_info.var_type != Type::Materia
                    && var_type != Type::Materia
                {
                    return Err(EvalError::InvalidOperation(
                        format!(
                            "Type mismatch: cannot assign {:?} to variable {} of type {:?}",
                            var_type, name, var_info.var_type
                        ),
                        line_info,
                    ));
                }

                var_info.value = value;
                return Ok(());
            }
        }
        Err(EvalError::UndefinedVariable(name.to_string(), line_info))
    }

    /// Registers a function in the current scope, associating it with its name.
    pub fn set_function(&mut self, name: String, function: Callable) {
        if let Some(current_scope) = self.function_scopes.last_mut() {
            current_scope.insert(name, function);
        }
    }

    /// Retrieves a function by name from the environment, searching from the most recent scope to the global scope.
    pub fn get_function(&self, name: &str) -> Option<&Callable> {
        for scope in self.function_scopes.iter().rev() {
            if let Some(function) = scope.get(name) {
                return Some(function);
            }
        }
        None
    }

    /// Registers multiple functions in the current scope at once.
    ///
    /// This method takes an iterator of (name, function) pairs and inserts them
    /// into the current function scope, providing a convenient way to batch-register functions.
    pub fn extend_functions<I>(&mut self, functions: I)
    where
        I: IntoIterator<Item = (String, Callable)>,
    {
        if let Some(scope) = self.function_scopes.last_mut() {
            for (name, callable) in functions {
                scope.insert(name, callable);
            }
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the value stored in a variable, which can be a boolean (Omen), integer (Arcana),
/// floating-point number (Aether), string (Rune), list (Scroll), or map (Lexicon).
#[derive(Debug, Clone)]
pub enum Value {
    Omen(bool),
    Arcana(i64),
    Aether(f64),
    Rune(String),
    Abyss,
    Scroll(Vec<Value>),
    Lexicon(HashMap<String, Value>),
}
