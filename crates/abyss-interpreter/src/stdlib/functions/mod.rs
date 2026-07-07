pub mod io;
pub mod math;
pub mod runtime;

use std::collections::HashMap;

use crate::env::{BuiltinFunction, Callable};

pub fn get_all_global_functions() -> HashMap<String, Callable> {
    let mut functions = HashMap::new();

    functions.insert(
        "unveil".to_string(),
        Callable::Builtin(BuiltinFunction {
            name: "unveil".to_string(),
            func: io::native_unveil,
        }),
    );

    functions.insert(
        "summon".to_string(),
        Callable::Builtin(BuiltinFunction {
            name: "summon".to_string(),
            func: io::native_summon,
        }),
    );

    for (name, func) in [
        ("perish", runtime::native_perish as crate::env::BuiltinFunc),
        ("abs", math::native_abs),
        ("sqrt", math::native_sqrt),
        ("floor", math::native_floor),
        ("ceil", math::native_ceil),
    ] {
        functions.insert(
            name.to_string(),
            Callable::Builtin(BuiltinFunction {
                name: name.to_string(),
                func,
            }),
        );
    }

    functions
}
