pub mod io;

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

    functions
}
