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

pub fn create_global_environment() -> RuntimeEnv {
    let mut env = RuntimeEnv::new();
    let functions = functions::get_all_global_functions();
    env.extend_functions(functions);
    let methods = methods::get_all_builtin_methods();
    env.set_builtin_methods(methods);
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
