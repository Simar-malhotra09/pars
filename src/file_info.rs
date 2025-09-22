use std::path::PathBuf;

#[derive(Debug)]
pub enum Language {
    Py,
    Rs,
    Unknown,
}

#[derive(Debug)]
pub struct FileInfo<'a> {
    pub file_type: Language,
    pub file_path: &'a PathBuf,
    pub file_size: usize,
}

impl<'a> FileInfo<'a> {
    pub fn from_path(path: &'a PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(path)?;

        let file_type = match path.extension().and_then(|ext| ext.to_str()) {
            Some("py") => Language::Py,
            Some("rs") => Language::Rs,
            _ => Language::Unknown,
        };

        Ok(FileInfo {
            file_type,
            file_path: path,
            file_size: metadata.len() as usize,
        })
    }
}
