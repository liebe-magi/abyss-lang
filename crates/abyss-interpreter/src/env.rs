use crate::diagnostics::{did_you_mean, label_with_suggestions};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{AST, LineInfo, Type};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type BuiltinFunc =
    fn(&mut RuntimeEnv, Vec<CallArg>, Option<LineInfo>) -> Result<EvalResult, EvalError>;

pub type BuiltinMethodHandler = fn(
    &mut RuntimeEnv,
    &AST,
    Option<&str>,
    Value,
    Vec<CallArg>,
    &Option<LineInfo>,
) -> Result<EvalResult, EvalError>;

pub type BuiltinMethodRegistry = HashMap<Type, HashMap<String, BuiltinMethodHandler>>;

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

#[derive(Debug, Clone)]
pub struct ArtifactMethod {
    pub function: EngravedFunction,
    pub requires_mutable_receiver: bool,
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
pub struct RuntimeEnv {
    scopes: Vec<HashMap<String, VarInfo>>, // Variable scopes
    function_scopes: Vec<HashMap<String, Callable>>, // Function scopes
    artifact_scopes: Vec<HashMap<String, ArtifactSchema>>, // Artifact schemas per scope
    builtin_methods: BuiltinMethodRegistry,
}

impl RuntimeEnv {
    /// Creates a new environment with an initial global scope.
    pub fn new() -> Self {
        RuntimeEnv {
            scopes: vec![HashMap::new()],
            function_scopes: vec![HashMap::new()],
            artifact_scopes: vec![HashMap::new()],
            builtin_methods: HashMap::new(),
        }
    }

    /// Pushes a new scope onto the stack, creating a new local environment for variables and functions.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.function_scopes.push(HashMap::new());
        self.artifact_scopes.push(HashMap::new());
    }

    /// Pops the most recent scope off the stack, discarding the current local environment.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
        self.function_scopes.pop();
        self.artifact_scopes.pop();
    }

    /// Returns the current variable-scope stack depth. Used today by the
    /// oracle scope-leak regression test in `eval::statements`; if future
    /// work adds analogous regression tests for `engrave` or `orbit`
    /// scopes they can reuse the same accessor. Test-only — exposing the
    /// depth in production builds would freeze it as part of the public
    /// API for no concrete user need today.
    #[cfg(test)]
    pub(crate) fn scope_depth(&self) -> usize {
        self.scopes.len()
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

    /// Build an `EvalError::UndefinedVariable` enriched with a "did you mean?"
    /// hint drawn from the variable scopes when a close lexical match exists.
    /// Falls back to the bare name when no plausible suggestion is available,
    /// so the message shape remains identical to the pre-suggestion era for
    /// truly unknown identifiers.
    pub fn undefined_variable_error(&self, name: &str, line_info: Option<LineInfo>) -> EvalError {
        let candidates: Vec<&str> = self
            .scopes
            .iter()
            .flat_map(|scope| scope.keys())
            .map(String::as_str)
            .collect();
        let suggestions = did_you_mean(name, candidates, 3);
        EvalError::UndefinedVariable(label_with_suggestions(name, &suggestions), line_info)
    }

    /// Same shape as [`undefined_variable_error`] but suggestions are drawn
    /// from the function scopes — used when an identifier appearing in a call
    /// position cannot be resolved to a `Callable`.
    pub fn undefined_function_error(&self, name: &str, line_info: Option<LineInfo>) -> EvalError {
        let candidates: Vec<&str> = self
            .function_scopes
            .iter()
            .flat_map(|scope| scope.keys())
            .map(String::as_str)
            .collect();
        let suggestions = did_you_mean(name, candidates, 3);
        EvalError::UndefinedVariable(label_with_suggestions(name, &suggestions), line_info)
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
        Err(self.undefined_variable_error(name, line_info))
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

    pub fn define_artifact(&mut self, schema: ArtifactSchema) -> Result<(), EvalError> {
        if let Some(scope) = self.artifact_scopes.last_mut() {
            if scope.contains_key(&schema.name) {
                return Err(EvalError::InvalidOperation(
                    format!("Artifact {} is already defined in this scope", schema.name),
                    schema.line_info.clone(),
                ));
            }
            scope.insert(schema.name.clone(), schema);
        }
        Ok(())
    }

    pub fn get_artifact(&self, name: &str) -> Option<&ArtifactSchema> {
        for scope in self.artifact_scopes.iter().rev() {
            if let Some(schema) = scope.get(name) {
                return Some(schema);
            }
        }
        None
    }

    pub fn get_artifact_mut(&mut self, name: &str) -> Option<&mut ArtifactSchema> {
        for scope in self.artifact_scopes.iter_mut().rev() {
            if let Some(schema) = scope.get_mut(name) {
                return Some(schema);
            }
        }
        None
    }

    pub fn artifact_defined_in_current_scope(&self, name: &str) -> bool {
        self.artifact_scopes
            .last()
            .map(|scope| scope.contains_key(name))
            .unwrap_or(false)
    }

    pub fn add_artifact_method(
        &mut self,
        artifact: &str,
        method_name: &str,
        method: ArtifactMethod,
        line_info: &Option<LineInfo>,
    ) -> Result<(), EvalError> {
        if let Some(schema) = self.get_artifact_mut(artifact) {
            if schema.methods.contains_key(method_name) {
                return Err(EvalError::InvalidOperation(
                    format!("Method {}::{} is already defined", artifact, method_name),
                    line_info.clone(),
                ));
            }
            schema.methods.insert(method_name.to_string(), method);
            Ok(())
        } else {
            Err(EvalError::InvalidOperation(
                format!("Artifact {} is not defined", artifact),
                line_info.clone(),
            ))
        }
    }

    pub fn get_artifact_method(&self, artifact: &str, method_name: &str) -> Option<ArtifactMethod> {
        self.get_artifact(artifact)
            .and_then(|schema| schema.method(method_name))
            .cloned()
    }

    pub fn set_builtin_methods(&mut self, methods: BuiltinMethodRegistry) {
        self.builtin_methods = methods;
    }

    pub fn builtin_methods(&self) -> &BuiltinMethodRegistry {
        &self.builtin_methods
    }
}

impl Default for RuntimeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the value stored in a variable, including primitive scalars, collections,
/// glyphs (type handles), and artifact instances.
#[derive(Debug, Clone)]
pub enum Value {
    Omen(bool),
    Arcana(i64),
    Aether(f64),
    Rune(Rc<String>),
    Abyss,
    Scroll(Rc<RefCell<Vec<Value>>>),
    Lexicon(Rc<RefCell<HashMap<String, Value>>>),
    Glyph(Type),
    Artifact(ArtifactHandle),
}

#[derive(Debug, Clone)]
pub struct ArtifactSchema {
    pub name: String,
    pub fields: Vec<ArtifactFieldSchema>,
    pub methods: HashMap<String, ArtifactMethod>,
    pub line_info: Option<LineInfo>,
}

impl ArtifactSchema {
    pub fn field(&self, name: &str) -> Option<&ArtifactFieldSchema> {
        self.fields.iter().find(|field| field.name == name)
    }

    pub fn field_names(&self) -> Vec<String> {
        self.fields.iter().map(|field| field.name.clone()).collect()
    }

    pub fn method(&self, name: &str) -> Option<&ArtifactMethod> {
        self.methods.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactFieldSchema {
    pub name: String,
    pub field_type: Type,
}

#[derive(Debug, Clone)]
pub struct ArtifactValue {
    pub type_name: String,
    pub fields: HashMap<String, Value>,
    pub field_order: Vec<String>,
}

pub type ArtifactHandle = Rc<RefCell<ArtifactValue>>;

#[cfg(test)]
mod tests {
    use super::*;

    fn rune(text: &str) -> Value {
        Value::Rune(Rc::new(text.to_string()))
    }

    fn artifact_schema(name: &str) -> ArtifactSchema {
        ArtifactSchema {
            name: name.to_string(),
            fields: Vec::new(),
            methods: HashMap::new(),
            line_info: None,
        }
    }

    fn engraved(name: &str) -> EngravedFunction {
        EngravedFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type: Type::Abyss,
            body: Box::new(AST::Abyss(None)),
            line_info: None,
        }
    }

    fn builtin(
        _: &mut RuntimeEnv,
        _: Vec<CallArg>,
        _: Option<LineInfo>,
    ) -> Result<EvalResult, EvalError> {
        Ok(EvalResult::abyss())
    }

    #[test]
    fn set_get_and_scope_resolution_of_vars() {
        let mut env = RuntimeEnv::new();
        env.set_var("sigil".into(), Value::Arcana(1), Type::Arcana, true, None);
        assert!(
            matches!(env.get_var("sigil"), Some(info) if matches!(info.value, Value::Arcana(1)))
        );

        env.push_scope();
        env.set_var("sigil".into(), Value::Arcana(5), Type::Arcana, false, None);
        assert!(
            matches!(env.get_var("sigil"), Some(info) if matches!(info.value, Value::Arcana(5)))
        );

        env.pop_scope();
        assert!(
            matches!(env.get_var("sigil"), Some(info) if matches!(info.value, Value::Arcana(1)))
        );
    }

    #[test]
    fn update_var_respects_mutability_and_types() {
        let mut env = RuntimeEnv::new();
        env.set_var("sigil".into(), Value::Arcana(1), Type::Arcana, false, None);
        let immutable_err = env
            .update_var("sigil", Value::Arcana(2), Type::Arcana, None)
            .unwrap_err();
        assert!(matches!(immutable_err, EvalError::InvalidOperation(_, _)));

        env.set_var("hex".into(), Value::Arcana(0), Type::Arcana, true, None);
        let type_err = env
            .update_var("hex", rune("glyph"), Type::Rune, None)
            .unwrap_err();
        assert!(matches!(type_err, EvalError::InvalidOperation(_, _)));

        env.set_var(
            "materia".into(),
            Value::Arcana(3),
            Type::Materia,
            true,
            None,
        );
        assert!(
            env.update_var("materia", rune("glyph"), Type::Rune, None)
                .is_ok()
        );

        let undefined_err = env
            .update_var("missing", Value::Arcana(0), Type::Arcana, None)
            .unwrap_err();
        assert!(matches!(undefined_err, EvalError::UndefinedVariable(_, _)));
    }

    #[test]
    fn extend_functions_registers_all_callable_entries() {
        let mut env = RuntimeEnv::new();
        env.extend_functions(vec![
            (
                "summon".into(),
                Callable::Builtin(BuiltinFunction {
                    name: "summon".into(),
                    func: builtin,
                }),
            ),
            ("engrave".into(), Callable::Engraved(engraved("engrave"))),
        ]);

        assert!(matches!(
            env.get_function("summon"),
            Some(Callable::Builtin(_))
        ));
        assert!(
            matches!(env.get_function("engrave"), Some(Callable::Engraved(func)) if func.name == "engrave")
        );
    }

    #[test]
    fn artifacts_can_be_defined_and_retrieved_per_scope() {
        let mut env = RuntimeEnv::new();
        env.define_artifact(artifact_schema("Relic")).unwrap();
        assert!(env.artifact_defined_in_current_scope("Relic"));

        env.push_scope();
        assert!(!env.artifact_defined_in_current_scope("Relic"));

        env.define_artifact(artifact_schema("Relic")).unwrap();
        assert!(env.artifact_defined_in_current_scope("Relic"));

        let duplicate = env.define_artifact(artifact_schema("Relic"));
        assert!(matches!(duplicate, Err(EvalError::InvalidOperation(_, _))));

        env.pop_scope();
        assert!(env.get_artifact("Relic").is_some());
    }

    #[test]
    fn add_artifact_method_registers_and_guards_duplicates() {
        let mut env = RuntimeEnv::new();
        env.define_artifact(artifact_schema("Relic")).unwrap();

        let method = ArtifactMethod {
            function: engraved("ignite"),
            requires_mutable_receiver: true,
        };
        env.add_artifact_method("Relic", "ignite", method.clone(), &None)
            .unwrap();

        let fetched = env.get_artifact_method("Relic", "ignite").unwrap();
        assert_eq!(fetched.function.name, "ignite");

        let duplicate = env.add_artifact_method("Relic", "ignite", method.clone(), &None);
        assert!(matches!(duplicate, Err(EvalError::InvalidOperation(_, _))));

        let missing = env.add_artifact_method("Unknown", "ignite", method, &None);
        assert!(matches!(missing, Err(EvalError::InvalidOperation(_, _))));
    }
    #[test]
    fn test_update_var_type_mismatch() {
        let mut env = RuntimeEnv::new();
        env.set_var("x".to_string(), Value::Arcana(1), Type::Arcana, true, None);

        let result = env.update_var("x", Value::Aether(1.0), Type::Aether, None);

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("Type mismatch")
        ));
    }

    #[test]
    fn test_update_var_immutable() {
        let mut env = RuntimeEnv::new();
        env.set_var("x".to_string(), Value::Arcana(1), Type::Arcana, false, None);

        let result = env.update_var("x", Value::Arcana(2), Type::Arcana, None);

        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("Cannot reassign to immutable variable")
        ));
    }

    #[test]
    fn test_define_artifact_conflict() {
        let mut env = RuntimeEnv::new();
        let schema = ArtifactSchema {
            name: "Player".to_string(),
            fields: Vec::new(),
            methods: HashMap::new(),
            line_info: None,
        };

        env.define_artifact(schema.clone())
            .expect("failed to define artifact");

        let result = env.define_artifact(schema);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("Artifact Player is already defined")
        ));
    }

    #[test]
    fn undefined_variable_error_includes_close_match_suggestion() {
        let mut env = RuntimeEnv::new();
        env.set_var(
            "counter".into(),
            Value::Arcana(0),
            Type::Arcana,
            false,
            None,
        );

        let err = env.undefined_variable_error("countar", None);
        match err {
            EvalError::UndefinedVariable(label, _) => {
                assert_eq!(label, "countar (did you mean: counter?)");
            }
            other => panic!("expected UndefinedVariable, got {:?}", other),
        }
    }

    #[test]
    fn undefined_variable_error_searches_outer_scopes_for_suggestions() {
        let mut env = RuntimeEnv::new();
        env.set_var(
            "outer_var".into(),
            Value::Arcana(0),
            Type::Arcana,
            false,
            None,
        );
        env.push_scope();
        env.set_var(
            "inner_var".into(),
            Value::Arcana(1),
            Type::Arcana,
            false,
            None,
        );

        // Typo close to outer_var; suggestion should still surface even
        // though the variable lives in a parent scope.
        let err = env.undefined_variable_error("outer_vra", None);
        match err {
            EvalError::UndefinedVariable(label, _) => {
                assert!(
                    label.contains("did you mean: outer_var"),
                    "expected outer_var suggestion, got {label:?}"
                );
            }
            other => panic!("expected UndefinedVariable, got {:?}", other),
        }
    }

    #[test]
    fn undefined_variable_error_falls_back_when_no_close_match() {
        let env = RuntimeEnv::new();
        // No variables at all — bare error message, no parenthesised hint.
        let err = env.undefined_variable_error("nothing_close", None);
        match err {
            EvalError::UndefinedVariable(label, _) => {
                assert_eq!(label, "nothing_close");
            }
            other => panic!("expected UndefinedVariable, got {:?}", other),
        }
    }

    #[test]
    fn undefined_function_error_searches_function_scopes() {
        let mut env = RuntimeEnv::new();
        env.extend_functions(vec![(
            "unveil".into(),
            Callable::Builtin(BuiltinFunction {
                name: "unveil".into(),
                func: builtin,
            }),
        )]);

        let err = env.undefined_function_error("unveels", None);
        match err {
            EvalError::UndefinedVariable(label, _) => {
                assert_eq!(label, "unveels (did you mean: unveil?)");
            }
            other => panic!("expected UndefinedVariable, got {:?}", other),
        }
    }
}
