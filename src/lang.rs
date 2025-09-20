pub trait LangSpec {
    const FUNC_DEF: &'static str;
    const PARAMS_OPEN: &'static str;
    const PARAMS_CLOSE: &'static str;
    const END_DEF: &'static str;

    fn is_valid_identifier(name: &str) -> bool;
}

pub mod py {
    use super::LangSpec;

    pub struct Python;

    impl LangSpec for Python {
        const FUNC_DEF: &'static str = "def";
        const PARAMS_OPEN: &'static str = "(";
        const PARAMS_CLOSE: &'static str = ")";
        const END_DEF: &'static str = ":";

        fn is_valid_identifier(name: &str) -> bool {
            name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        }
    }
}

pub mod rs {
    use super::LangSpec;

    pub struct Rust;

    impl LangSpec for Rust {
        const FUNC_DEF: &'static str = "fn";
        const PARAMS_OPEN: &'static str = "(";
        const PARAMS_CLOSE: &'static str = ")";
        const END_DEF: &'static str = "{";

        fn is_valid_identifier(name: &str) -> bool {
            // Very simplified Rust check
            name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_alphanumeric() || c == '_')
        }
    }
}
