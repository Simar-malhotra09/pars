use crate::cli::Cli;

#[derive(Debug)]
pub struct Config {
    pub enable_cache: bool,
}

impl From<&Cli> for Config {
    fn from(cli: &Cli) -> Self {
        Self {
            enable_cache: !cli.no_cache,
        }
    }
}
