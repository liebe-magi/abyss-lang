use crate::env::{CallArg, RuntimeEnv, Value};
use crate::eval::{EvalError, EvalResult};
use crate::io_bridge::IoBridge;
use abyss_core::ast::{Span, Type};
use std::rc::Rc;

pub fn native_unveil(
    env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line: Option<Span>,
) -> Result<EvalResult, EvalError> {
    // Route writes through the bridge `RuntimeEnv` carries, so non-CLI
    // hosts (Wasm playground, future LSP) can capture output without
    // touching `stdout`. CLI builds default to `StdIoBridge`.
    native_unveil_with_io(args, line, env.io_bridge_mut())
}

fn map_io_error(message: &str, line: &Option<Span>) -> EvalError {
    EvalError::InvalidOperation(message.to_string(), *line)
}

pub(crate) fn native_unveil_with_io(
    args: Vec<CallArg>,
    line: Option<Span>,
    io: &mut dyn IoBridge,
) -> Result<EvalResult, EvalError> {
    if args.is_empty() {
        return Err(EvalError::InvalidOperation(
            "unveil() requires at least 1 argument".to_string(),
            line,
        ));
    }

    let outputs: Result<Vec<String>, EvalError> = args
        .iter()
        .map(|arg| format_eval_result(&arg.value, &line))
        .collect();

    let output_str = outputs?.join("");
    io.write_str(&output_str)
        .map_err(|_| map_io_error("Failed to write unveil output", &line))?;
    io.write_str("\n")
        .map_err(|_| map_io_error("Failed to write unveil newline", &line))?;
    Ok(EvalResult::abyss())
}

pub fn native_summon(
    env: &mut RuntimeEnv,
    args: Vec<CallArg>,
    line: Option<Span>,
) -> Result<EvalResult, EvalError> {
    native_summon_with_io(args, line, env.io_bridge_mut())
}

pub(crate) fn native_summon_with_io(
    args: Vec<CallArg>,
    line: Option<Span>,
    io: &mut dyn IoBridge,
) -> Result<EvalResult, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::InvalidOperation(
            "summon() requires exactly 1 argument (prompt)".to_string(),
            line,
        ));
    }

    let prompt = match &args[0].value {
        EvalResult::Data(Value::Rune(r)) => r.as_ref(),
        _ => {
            return Err(EvalError::TypeError(
                "summon() argument must be a Rune (prompt)".to_string(),
                line,
            ));
        }
    };

    io.write_str(prompt)
        .and_then(|_| io.flush())
        .map_err(|_| map_io_error("Failed to flush stdout", &line))?;

    let mut input = String::new();
    // Preserve the underlying `io::Error` message so a bridge that
    // refuses reads (e.g. the Wasm Playground bridge, which has no
    // interactive stdin) can surface its specific reason to the user
    // instead of being collapsed into a generic "Failed to read input".
    io.read_line(&mut input)
        .map_err(|err| map_io_error(&format!("Failed to read input: {}", err), &line))?;

    Ok(EvalResult::data(Value::Rune(Rc::new(
        input.trim().to_string(),
    ))))
}

pub(crate) fn format_eval_result(
    value: &EvalResult,
    line: &Option<Span>,
) -> Result<String, EvalError> {
    match value {
        EvalResult::Data(Value::Artifact(handle)) => format_artifact(handle, line),
        EvalResult::Data(inner) => format_value(inner, line),
        EvalResult::Revealed(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil a Revealed value (control flow construct)".to_string(),
            *line,
        )),
        EvalResult::Revolve(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil a Revolve value (control flow construct)".to_string(),
            *line,
        )),
        EvalResult::Eject(_) => Err(EvalError::InvalidOperation(
            "Cannot unveil an Eject value (control flow construct)".to_string(),
            *line,
        )),
    }
}

fn glyph_label(var_type: &Type) -> String {
    match var_type {
        Type::Arcana => "arcana".to_string(),
        Type::Aether => "aether".to_string(),
        Type::Rune => "rune".to_string(),
        Type::Omen => "omen".to_string(),
        Type::Abyss => "abyss".to_string(),
        Type::Scroll => "scroll".to_string(),
        Type::Lexicon => "lexicon".to_string(),
        Type::Materia => "materia".to_string(),
        Type::Glyph => "glyph".to_string(),
        Type::Fate => "fate".to_string(),
        Type::Augury => "augury".to_string(),
        Type::Artifact(name) => name.clone(),
    }
}

pub(crate) fn format_value(value: &Value, line: &Option<Span>) -> Result<String, EvalError> {
    match value {
        Value::Omen(b) => Ok(if *b { "boon" } else { "hex" }.to_string()),
        Value::Arcana(n) => Ok(n.to_string()),
        Value::Aether(n) => Ok(n.to_string()),
        Value::Rune(s) => Ok(s.replace("\\n", "\n")),
        Value::Abyss => Ok(String::new()),
        Value::Scroll(items) => {
            let parts: Result<Vec<String>, EvalError> = items
                .borrow()
                .iter()
                .map(|item| format_value(item, line))
                .collect();
            Ok(format!("[{}]", parts?.join(", ")))
        }
        Value::Lexicon(entries) => {
            let mut pieces = Vec::new();
            for (key, val) in entries.borrow().iter() {
                let formatted_value = format_value(val, line)?;
                pieces.push(format!("\"{}\": {}", key, formatted_value));
            }
            Ok(format!("{{{}}}", pieces.join(", ")))
        }
        Value::Glyph(var_type) => Ok(glyph_label(var_type)),
        Value::Artifact(handle) => format_artifact(handle, line),
    }
}

pub(crate) fn format_artifact(
    handle: &crate::env::ArtifactHandle,
    line: &Option<Span>,
) -> Result<String, EvalError> {
    let borrowed = handle.borrow();
    let mut pieces = Vec::new();
    for field in &borrowed.field_order {
        if let Some(value) = borrowed.fields.get(field) {
            let formatted = format_value(value, line)?;
            pieces.push(format!("{}: {}", field, formatted));
        }
    }
    Ok(format!(
        "{} {{ {} }}",
        borrowed.type_name,
        pieces.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::{ArtifactHandle, ArtifactValue};
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::io;
    use std::rc::Rc;

    #[derive(Default)]
    struct MockIo {
        writes: Vec<String>,
        reads: VecDeque<Result<String, &'static str>>,
        fail_write: bool,
        fail_flush: bool,
    }

    impl MockIo {
        fn with_reads(lines: &[&str]) -> Self {
            let mut io = MockIo::default();
            for line in lines {
                io.reads.push_back(Ok((*line).to_string()));
            }
            io
        }

        fn with_write_failure() -> Self {
            MockIo {
                fail_write: true,
                ..Default::default()
            }
        }

        fn with_flush_failure() -> Self {
            MockIo {
                fail_flush: true,
                ..Default::default()
            }
        }

        fn push_error(&mut self, msg: &'static str) {
            self.reads.push_back(Err(msg));
        }
    }

    impl IoBridge for MockIo {
        fn write_str(&mut self, content: &str) -> io::Result<()> {
            if self.fail_write {
                return Err(io::Error::other("write failure"));
            }
            self.writes.push(content.to_string());
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            if self.fail_flush {
                return Err(io::Error::other("flush failure"));
            }
            Ok(())
        }

        fn read_line(&mut self, buffer: &mut String) -> io::Result<()> {
            match self.reads.pop_front() {
                Some(Ok(line)) => {
                    buffer.push_str(&line);
                    Ok(())
                }
                Some(Err(msg)) => Err(io::Error::other(msg)),
                None => Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "no input queued",
                )),
            }
        }
    }

    fn arg(value: EvalResult) -> CallArg {
        CallArg {
            value,
            var_name: None,
        }
    }

    fn artifact_handle() -> ArtifactHandle {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::Rune(Rc::new("Alya".to_string())));
        fields.insert("level".to_string(), Value::Arcana(7));
        let field_order = vec!["name".to_string(), "level".to_string()];
        Rc::new(RefCell::new(ArtifactValue {
            type_name: "Player".to_string(),
            fields,
            field_order,
        }))
    }

    #[test]
    fn native_unveil_with_io_writes_all_arguments() {
        let args = vec![
            arg(EvalResult::data(Value::Omen(true))),
            arg(EvalResult::data(Value::Rune(Rc::new("hex".to_string())))),
        ];
        let mut io = MockIo::default();
        let result = native_unveil_with_io(args, None, &mut io).expect("unveil should succeed");
        assert!(matches!(result, EvalResult::Data(Value::Abyss)));
        assert_eq!(io.writes.join(""), "boonhex\n");
    }

    #[test]
    fn native_unveil_with_io_rejects_control_flow() {
        let args = vec![arg(EvalResult::Revealed(Box::new(EvalResult::abyss())))];
        let mut io = MockIo::default();
        let err = native_unveil_with_io(args, None, &mut io)
            .expect_err("control flow values should error");
        if let EvalError::InvalidOperation(msg, _) = err {
            assert!(msg.contains("Revealed"));
        } else {
            panic!("expected invalid operation, got {:?}", err);
        }
    }

    #[test]
    fn native_unveil_with_io_propagates_write_errors() {
        let args = vec![arg(EvalResult::data(Value::Arcana(1)))];
        let mut io = MockIo::with_write_failure();
        assert!(native_unveil_with_io(args, None, &mut io).is_err());
    }

    #[test]
    fn native_summon_with_io_returns_trimmed_input() {
        let args = vec![arg(EvalResult::data(Value::Rune(Rc::new("?".to_string()))))];
        let mut io = MockIo::with_reads(&["mage answer\n"]);
        let result = native_summon_with_io(args, None, &mut io).expect("summon should succeed");
        match result {
            EvalResult::Data(Value::Rune(text)) => assert_eq!(text.as_ref(), "mage answer"),
            other => panic!("expected rune result, got {:?}", other),
        }
        assert_eq!(io.writes.join(""), "?");
    }

    #[test]
    fn native_summon_with_io_errors_on_read_failure() {
        let args = vec![arg(EvalResult::data(Value::Rune(Rc::new("?".to_string()))))];
        let mut io = MockIo::with_reads(&[]);
        io.push_error("read failure");
        assert!(native_summon_with_io(args, None, &mut io).is_err());
    }

    #[test]
    fn native_summon_with_io_errors_on_flush_failure() {
        let args = vec![arg(EvalResult::data(Value::Rune(Rc::new("?".to_string()))))];
        let mut io = MockIo::with_flush_failure();
        assert!(native_summon_with_io(args, None, &mut io).is_err());
    }

    #[test]
    fn format_value_handles_collections_and_artifacts() {
        let scroll = Value::Scroll(Rc::new(RefCell::new(vec![
            Value::Arcana(1),
            Value::Arcana(2),
        ])));
        let mut lex_entries = HashMap::new();
        lex_entries.insert("alpha".to_string(), Value::Omen(true));
        let lexicon = Value::Lexicon(Rc::new(RefCell::new(lex_entries)));
        let artifact = Value::Artifact(artifact_handle());

        assert_eq!(format_value(&scroll, &None).unwrap(), "[1, 2]");
        assert!(
            format_value(&lexicon, &None)
                .unwrap()
                .contains("\"alpha\": boon")
        );
        assert!(
            format_value(&artifact, &None)
                .unwrap()
                .starts_with("Player {")
        );
    }

    #[test]
    fn native_unveil_requires_arguments() {
        let args = vec![];
        let mut io = MockIo::default();
        let result = native_unveil_with_io(args, None, &mut io);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires at least 1 argument")
        ));
    }

    #[test]
    fn native_summon_requires_exactly_one_argument() {
        let args = vec![];
        let mut io = MockIo::default();
        let result = native_summon_with_io(args, None, &mut io);
        assert!(matches!(
            result,
            Err(EvalError::InvalidOperation(msg, _)) if msg.contains("requires exactly 1 argument")
        ));
    }

    #[test]
    fn native_summon_requires_rune_argument() {
        let args = vec![arg(EvalResult::data(Value::Arcana(1)))];
        let mut io = MockIo::default();
        let result = native_summon_with_io(args, None, &mut io);
        assert!(matches!(
            result,
            Err(EvalError::TypeError(msg, _)) if msg.contains("must be a Rune")
        ));
    }
}
