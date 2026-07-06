use crate::diagnostics::{did_you_mean, label_with_suggestions};
use crate::eval::{EvalError, EvalResult};
use abyss_core::ast::{EngraveParam, Expr, Span, Stmt, Type};
use std::collections::HashMap;

// Value and artifact types moved to dedicated modules; re-exported here so
// pre-split paths (`crate::env::Value`, `abyss_interpreter::env::Value`, …)
// keep working.
pub use crate::artifact::{
    ArtifactFieldSchema, ArtifactHandle, ArtifactMethod, ArtifactSchema, ArtifactValue,
};
pub use crate::value::Value;

pub type BuiltinFunc =
    fn(&mut RuntimeEnv, Vec<CallArg>, Option<Span>) -> Result<EvalResult, EvalError>;

pub type BuiltinMethodHandler = fn(
    &mut RuntimeEnv,
    &Expr,
    Option<&str>,
    Value,
    Vec<CallArg>,
    &Option<Span>,
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
    pub params: Vec<EngraveParam>,
    pub return_type: Type,
    pub body: Box<Stmt>,
    pub line_info: Option<Span>,
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
    pub line_info: Option<Span>,
}

/// Manages variable and function scopes in the execution environment, including
/// both the global scope and any nested local scopes. Carries the
/// [`IoBridge`](crate::io_bridge::IoBridge) used by `unveil` / `summon`,
/// so non-CLI hosts (Wasm playground, future LSP) can swap in a custom
/// implementation that captures output instead of touching `stdout`.
pub struct RuntimeEnv {
    scopes: Vec<HashMap<String, VarInfo>>, // Variable scopes
    function_scopes: Vec<HashMap<String, Callable>>, // Function scopes
    artifact_scopes: Vec<HashMap<String, ArtifactSchema>>, // Artifact schemas per scope
    builtin_methods: BuiltinMethodRegistry,
    io_bridge: Box<dyn crate::io_bridge::IoBridge>,
}

impl RuntimeEnv {
    /// Creates a new environment with an initial global scope and the
    /// default [`StdIoBridge`](crate::io_bridge::StdIoBridge), so CLI /
    /// REPL paths get the same `stdout` / `stdin` behaviour they had
    /// before the bridge abstraction landed. Non-CLI hosts call
    /// [`set_io_bridge`](Self::set_io_bridge) afterwards to swap in
    /// their own implementation.
    pub fn new() -> Self {
        RuntimeEnv {
            scopes: vec![HashMap::new()],
            function_scopes: vec![HashMap::new()],
            artifact_scopes: vec![HashMap::new()],
            builtin_methods: HashMap::new(),
            io_bridge: Box::new(crate::io_bridge::StdIoBridge),
        }
    }

    /// Replace the I/O bridge used by `unveil` / `summon`. Wasm callers
    /// install a `String`-backed implementation here so output the
    /// playground would otherwise drop to `stdout` is captured into a
    /// buffer the host can hand back to JavaScript.
    pub fn set_io_bridge(&mut self, bridge: Box<dyn crate::io_bridge::IoBridge>) {
        self.io_bridge = bridge;
    }

    /// Mutable access to the installed I/O bridge. Used by
    /// `stdlib::functions::io` to route `unveil` writes and `summon`
    /// reads through the bridge the host provided.
    pub fn io_bridge_mut(&mut self) -> &mut dyn crate::io_bridge::IoBridge {
        self.io_bridge.as_mut()
    }

    /// Pushes a new scope onto the stack, creating a new local environment for variables and functions.
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.function_scopes.push(HashMap::new());
        self.artifact_scopes.push(HashMap::new());
    }

    /// Current scope-stack depth. Paired with [`truncate_scopes`](Self::truncate_scopes)
    /// so `?`-propagation catch points can restore the exact depth they saw,
    /// even when the propagation unwound out of nested `orbit` / `oracle`
    /// scopes that never reached their own pops.
    pub(crate) fn scopes_len(&self) -> usize {
        self.scopes.len()
    }

    /// Pop scopes until the stack is back at `depth`.
    pub(crate) fn truncate_scopes(&mut self, depth: usize) {
        while self.scopes.len() > depth {
            self.pop_scope();
        }
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
        line_info: Option<Span>,
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
    pub fn undefined_variable_error(&self, name: &str, line_info: Option<Span>) -> EvalError {
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
    pub fn undefined_function_error(&self, name: &str, line_info: Option<Span>) -> EvalError {
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
        line_info: Option<Span>,
    ) -> Result<(), EvalError> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var_info) = scope.get_mut(name) {
                if !var_info.is_morph {
                    return Err(EvalError::ImmutableAssignment(name.to_string(), line_info));
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

    /// Names claimed by the error-handling built-ins. User artifacts may
    /// not use them in any scope, so `bless { … }` always means the seeded
    /// variant and patterns can never be shadowed into silence.
    pub const RESERVED_ARTIFACT_NAMES: [&'static str; 6] =
        ["fate", "augury", "bless", "curse", "manifest", "naught"];

    /// Registers a built-in artifact schema, bypassing the reserved-name
    /// guard. Only the stdlib seeding path uses this.
    pub(crate) fn define_builtin_artifact(&mut self, schema: ArtifactSchema) {
        if let Some(scope) = self.artifact_scopes.first_mut() {
            scope.insert(schema.name.clone(), schema);
        }
    }

    pub fn define_artifact(&mut self, schema: ArtifactSchema) -> Result<(), EvalError> {
        if Self::RESERVED_ARTIFACT_NAMES.contains(&schema.name.as_str()) {
            return Err(EvalError::InvalidOperation(
                format!(
                    "Artifact name {} is reserved by the error-handling built-ins",
                    schema.name
                ),
                schema.line_info,
            ));
        }
        if let Some(scope) = self.artifact_scopes.last_mut() {
            if scope.contains_key(&schema.name) {
                return Err(EvalError::InvalidOperation(
                    format!("Artifact {} is already defined in this scope", schema.name),
                    schema.line_info,
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
        line_info: &Option<Span>,
    ) -> Result<(), EvalError> {
        if let Some(schema) = self.get_artifact_mut(artifact) {
            if schema.methods.contains_key(method_name) {
                return Err(EvalError::InvalidOperation(
                    format!("Method {}::{} is already defined", artifact, method_name),
                    *line_info,
                ));
            }
            schema.methods.insert(method_name.to_string(), method);
            Ok(())
        } else {
            Err(EvalError::InvalidOperation(
                format!("Artifact {} is not defined", artifact),
                *line_info,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::test_support::rune;

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
            body: Box::new(Stmt::Expr(Expr::Abyss(None), None)),
            line_info: None,
        }
    }

    fn builtin(
        _: &mut RuntimeEnv,
        _: Vec<CallArg>,
        _: Option<Span>,
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
        assert!(
            matches!(immutable_err, EvalError::ImmutableAssignment(name, _) if name == "sigil")
        );

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
            Err(EvalError::ImmutableAssignment(name, _)) if name == "x"
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
