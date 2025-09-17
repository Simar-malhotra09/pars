use std::path::PathBuf;
use clap::{Parser, ValueEnum};


#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum InfoLevel {
    L1,
    L2,
    L3,
}

#[derive(Parser, Debug)]
pub struct Cli {
    pub file_path: PathBuf,

    #[clap(value_enum, default_value_t = InfoLevel::L1)]
    pub info_level: InfoLevel,
    
    /// Number of threads for parallel processing if enabled 
    #[clap(long, default_value_t = 8)]
    pub threads: usize,
    
    /// Block size in KB for parallel reading if enabled
    #[clap(long, default_value_t = 16)]
    pub block_size_kb: usize,
    
    #[clap(long)]
    pub no_cache: bool,
    
    #[clap(long)]
    pub parallel_read: bool,
}


