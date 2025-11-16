pub mod io;

use crate::ast::Type;
use crate::env::{BuiltinFunction, Callable, Environment, Value};
use std::collections::HashMap;

fn get_all_builtins() -> HashMap<String, Callable> {
    let mut builtins = HashMap::new();

    builtins.insert(
        "unveil".to_string(),
        Callable::Builtin(BuiltinFunction {
            name: "unveil".to_string(),
            func: io::native_unveil,
        }),
    );

    builtins.insert(
        "summon".to_string(),
        Callable::Builtin(BuiltinFunction {
            name: "summon".to_string(),
            func: io::native_summon,
        }),
    );

    builtins
}

fn seed_builtin_glyphs(env: &mut Environment) {
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

pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    let builtins = get_all_builtins();
    env.extend_functions(builtins);
    seed_builtin_glyphs(&mut env);
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
