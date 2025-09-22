#[derive(Debug)]
pub enum ParseError {
    IoError(std::io::Error),
    CacheError(String),
    ParseFailure(String),
    UnsupportedLanguage(String),
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::IoError(err)
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(e) => write!(f, "IO error: {}", e),
            ParseError::CacheError(e) => write!(f, "Cache error: {}", e),
            ParseError::ParseFailure(e) => write!(f, "Parse error: {}", e),
            ParseError::UnsupportedLanguage(e) => write!(f, "Language is not supported yet: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}
