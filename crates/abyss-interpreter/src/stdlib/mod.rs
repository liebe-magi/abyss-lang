pub mod functions;
pub mod methods;

use crate::env::{RuntimeEnv, Value};
use abyss_core::ast::Type;

fn seed_builtin_glyphs(env: &mut RuntimeEnv) {
    let glyphs = [
        ("arcana", Type::Arcana),
        ("aether", Type::Aether),
        ("rune", Type::Rune),
        ("omen", Type::Omen),
        ("abyss", Type::Abyss),
        ("scroll", Type::Scroll),
        ("lexicon", Type::Lexicon),
        ("materia", Type::Materia),
        ("glyph", Type::Glyph),
    ];

    for (name, glyph_type) in glyphs {
        env.set_var(
            name.to_string(),
            Value::Glyph(glyph_type.clone()),
            Type::Glyph,
            false,
            None,
        );
    }
}

/// Seed the error-handling variant artifacts (`bless` / `curse` for
/// `fate`, `manifest` / `naught` for `augury`) plus their glyph
/// variables, mirroring what an `artifact` definition would create.
/// Payload fields are `materia` until generics land.
fn seed_error_handling_artifacts(env: &mut RuntimeEnv) {
    use crate::artifact::{ArtifactFieldSchema, ArtifactSchema};
    use std::collections::HashMap;

    let variants: [(&str, Option<&str>); 4] = [
        ("bless", Some("value")),
        ("curse", Some("reason")),
        ("manifest", Some("value")),
        ("naught", None),
    ];
    for (name, field) in variants {
        let fields = field
            .map(|f| {
                vec![ArtifactFieldSchema {
                    name: f.to_string(),
                    field_type: Type::Materia,
                }]
            })
            .unwrap_or_default();
        env.define_builtin_artifact(ArtifactSchema {
            name: name.to_string(),
            fields,
            methods: HashMap::new(),
            line_info: None,
        });
        env.set_var(
            name.to_string(),
            Value::Glyph(Type::Artifact(name.to_string())),
            Type::Glyph,
            false,
            None,
        );
    }
}

/// Construct one of the error-handling variant artifacts from Rust —
/// the stdlib's counterpart of writing `bless {{ value: … }}` in AbySS.
pub(crate) fn make_variant(name: &str, field: Option<(&str, Value)>) -> Value {
    use crate::artifact::ArtifactValue;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    let mut fields = HashMap::new();
    let mut field_order = Vec::new();
    if let Some((field_name, value)) = field {
        field_order.push(field_name.to_string());
        fields.insert(field_name.to_string(), value);
    }
    Value::Artifact(Rc::new(RefCell::new(ArtifactValue {
        type_name: name.to_string(),
        fields,
        field_order,
    })))
}

pub fn create_global_environment() -> RuntimeEnv {
    let mut env = RuntimeEnv::new();
    let functions = functions::get_all_global_functions();
    env.extend_functions(functions);
    let methods = methods::get_all_builtin_methods();
    env.set_builtin_methods(methods);
    seed_builtin_glyphs(&mut env);
    seed_error_handling_artifacts(&mut env);
    env
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_glyphs_are_seeded() {
        let env = create_global_environment();
        for glyph in [
            "arcana", "aether", "rune", "omen", "abyss", "scroll", "lexicon", "materia", "glyph",
        ] {
            let entry = env
                .get_var(glyph)
                .unwrap_or_else(|| panic!("missing glyph {}", glyph));
            assert!(matches!(entry.value, Value::Glyph(_)));
            assert_eq!(entry.var_type, Type::Glyph);
            assert!(!entry.is_morph);
        }
    }
}
