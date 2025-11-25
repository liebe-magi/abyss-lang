#[cfg(test)]
mod tests {
    use crate::env::{RuntimeEnv, Value};
    use crate::eval::{EvalError, EvalResult, evaluate};
    use abyss_core::parser::parse;

    fn run_code(src: &str, env: &mut RuntimeEnv) -> Result<EvalResult, EvalError> {
        let outcome = parse(src);
        if !outcome.diagnostics.is_empty() {
            panic!("Parse error: {:?}", outcome.diagnostics);
        }

        let mut last_result = EvalResult::abyss();
        for node in outcome.ast {
            last_result = evaluate(&node, env)?;
        }
        Ok(last_result)
    }

    #[test]
    fn test_spectrum_definition() {
        let src = "spectrum Color { Red, Green, Blue };";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let schema = env
            .get_spectrum("Color")
            .expect("Color spectrum should be defined");
        assert!(schema.variants.contains_key("Red"));
        assert!(schema.variants.contains_key("Green"));
        assert!(schema.variants.contains_key("Blue"));
    }

    #[test]
    fn test_spectrum_instantiation() {
        let src = "
            spectrum Color { Red, Green, Blue };
            forge c: Color = Color::Red;
        ";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let c = env.get_var("c").unwrap();
        if let Value::Spectrum {
            name,
            variant,
            data,
        } = &c.value
        {
            assert_eq!(name, "Color");
            assert_eq!(variant, "Red");
            assert!(data.is_empty());
        } else {
            panic!("Expected Spectrum value");
        }
    }

    #[test]
    fn test_spectrum_tuple_variant() {
        let src = "
            spectrum Result { Ok(arcana), Err(rune) };
            forge res: Result = Result::Ok(42);
        ";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let res = env.get_var("res").unwrap();
        if let Value::Spectrum {
            name,
            variant,
            data,
        } = &res.value
        {
            assert_eq!(name, "Result");
            assert_eq!(variant, "Ok");
            assert_eq!(data.len(), 1);
            if let Value::Arcana(v) = data[0] {
                assert_eq!(v, 42);
            } else {
                panic!("Expected Arcana data");
            }
        } else {
            panic!("Expected Spectrum value");
        }
    }

    #[test]
    fn test_oracle_matching() {
        let src = "
            spectrum Color { Red, Green, Blue };
            forge c: Color = Color::Green;
            forge result: arcana = oracle (c) {
                Color::Red => 1;
                Color::Green => 2;
                Color::Blue => 3;
            };
        ";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let result = env.get_var("result").unwrap();
        if let Value::Arcana(v) = result.value {
            assert_eq!(v, 2);
        } else {
            panic!("Expected Arcana result");
        }
    }

    #[test]
    fn test_oracle_destructuring() {
        let src = "
            spectrum Result { Ok(arcana), Err(rune) };
            forge res: Result = Result::Ok(42);
            forge val: arcana = oracle (res) {
                Result::Ok(x) => x;
                Result::Err(_) => 0;
            };
        ";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let val = env.get_var("val").unwrap();
        if let Value::Arcana(v) = val.value {
            assert_eq!(v, 42);
        } else {
            panic!("Expected Arcana result");
        }
    }

    #[test]
    fn test_exhaustiveness_with_wildcard() {
        let src = "
            spectrum Color { Red, Green, Blue };
            forge c: Color = Color::Blue;
            forge result: arcana = oracle (c) {
                Color::Red => 1;
                _ => 0;
            };
        ";
        let mut env = RuntimeEnv::new();
        run_code(src, &mut env).unwrap();

        let result = env.get_var("result").unwrap();
        if let Value::Arcana(v) = result.value {
            assert_eq!(v, 0);
        } else {
            panic!("Expected Arcana result");
        }
    }
}
