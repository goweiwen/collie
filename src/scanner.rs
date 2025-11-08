use tracing::info;

use crate::console::{Console, ConsolesConfig};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RomFile {
    pub path: PathBuf,
    pub name: String,
    pub name_no_extension: String,
    pub console: Console,
}

pub struct RomScanner {
    consoles_config: ConsolesConfig,
}

impl RomScanner {
    pub fn new(consoles_config: ConsolesConfig) -> Self {
        Self { consoles_config }
    }

    /// Scan a directory for ROM files
    pub fn scan_directory(&self, roms_path: &Path) -> Result<Vec<RomFile>, std::io::Error> {
        let mut rom_files = Vec::new();

        // Read all directories in the ROMs path
        for entry in fs::read_dir(roms_path)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let folder_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // Check if this folder matches a known console
            if let Some(console) = self.consoles_config.find_console(&folder_name) {
                info!("Scanning {} ({})...", folder_name, console.name);

                // Scan ROMs in this console folder
                let console_roms = self.scan_console_folder(&path, console)?;
                rom_files.extend(console_roms);
            }
        }

        Ok(rom_files)
    }

    /// Scan a specific console folder for ROM files
    fn scan_console_folder(
        &self,
        console_path: &Path,
        console: &Console,
    ) -> Result<Vec<RomFile>, std::io::Error> {
        let mut rom_files = Vec::new();

        for entry in fs::read_dir(console_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip directories, Imgs folder, and hidden files
            if path.is_dir()
                || path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.') || n == "Imgs")
                    .unwrap_or(true)
            {
                continue;
            }

            // Skip certain file types
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();

                // Skip non-ROM files
                if ext_lower == "xml"
                    || ext_lower == "miyoocmd"
                    || ext_lower == "cfg"
                    || ext_lower == "db"
                    || ext_lower == "nfo"
                {
                    continue;
                }
            }

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let name_no_extension = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            rom_files.push(RomFile {
                path: path.clone(),
                name,
                name_no_extension,
                console: console.clone(),
            });
        }

        Ok(rom_files)
    }
}
