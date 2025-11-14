pub mod io;

use crate::env::{BuiltinFunction, Callable, Environment};
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

pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    let builtins = get_all_builtins();
    env.extend_functions(builtins);
    env
}
