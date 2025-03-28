use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct Initializer {
    data_dir: String,
}

impl Initializer {
    pub fn new(data_dir: &str) -> Self {
        Self {
            data_dir: data_dir.to_string(),
        }
    }

    pub fn init(&self) -> Result<()> {
        // Create data directory if it doesn't exist
        if !Path::new(&self.data_dir).exists() {
            fs::create_dir_all(&self.data_dir)?;
        }

        // Initialize services.json if it doesn't exist
        let services_file = format!("{}/services.json", self.data_dir);
        if !Path::new(&services_file).exists() {
            fs::write(&services_file, "[]")?;
        }

        // Initialize users.json if it doesn't exist
        let users_file = format!("{}/users.json", self.data_dir);
        if !Path::new(&users_file).exists() {
            fs::write(&users_file, "[]")?;
        }

        Ok(())
    }
} 