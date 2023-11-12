use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::io::Write;
use std::fs::File;
use std::io;

#[derive(Debug)]
pub struct GitConfig {
    core: HashMap<String, String>,
    remote_origin: HashMap<String, String>,
    branch_main: HashMap<String, String>,
}

impl GitConfig {
    pub fn new() -> Self {
        Self {
            core: HashMap::new(),
            remote_origin: HashMap::new(),
            branch_main: HashMap::new(),
        }
    }

    pub fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "repositoryformatversion" | "filemode" | "bare" | "logallrefupdates" => {
                    self.core.insert(key.to_string(), value.to_string());
                }
                "url" | "fetch" => {
                    self.remote_origin.insert(key.to_string(), value.to_string());
                }
                "remote" | "merge" => {
                    self.branch_main.insert(key.to_string(), value.to_string());
                }
                _ => {}
            }
        }
    }

    pub fn read_git_config(&mut self, file_path: &str) -> Result<(), std::io::Error> {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path)?;

            for line in content.lines() {
                self.parse_line(line);
            }
        }

        Ok(())
    }

    pub fn add_entry(&mut self, key: &str, value: &str, section: &str) {
        match section {
            "core" => {
                self.core.insert(key.to_string(), value.to_string());
            }
            "remote.origin" => {
                self.remote_origin.insert(key.to_string(), value.to_string());
            }
            "branch.main" => {
                self.branch_main.insert(key.to_string(), value.to_string());
            }
            _ => {}
        }
    }

    pub fn get_value(&self, key: &str, section: &str) -> Option<&String> {
        match section {
            "core" => self.core.get(key),
            "remote.origin" => self.remote_origin.get(key),
            "branch.main" => self.branch_main.get(key),
            _ => None,
        }
    }

    pub fn write_to_file(&self, file_path: &str) -> io::Result<()> {
        let mut file = File::create(file_path)?;

        // Write core section
        for (key, value) in &self.core {
            writeln!(file, "{} = {}", key, value)?;
        }

        // Write remote "origin" section
        for (key, value) in &self.remote_origin {
            writeln!(file, "[remote \"origin\"]")?;
            writeln!(file, "    {} = {}", key, value)?;
        }

        // Write branch "main" section
        for (key, value) in &self.branch_main {
            writeln!(file, "[branch \"main\"]")?;
            writeln!(file, "    {} = {}", key, value)?;
        }

        Ok(())
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn parse_line_valid_core_key_value() {
        let mut git_config = GitConfig::new();
        git_config.parse_line("repositoryformatversion = 0");
        assert_eq!(git_config.core.get("repositoryformatversion"), Some(&"0".to_string()));
    }

    #[test]
    fn parse_line_valid_remote_origin_key_value() {
        let mut git_config = GitConfig::new();
        git_config.parse_line("url = git@github.com:example/repo.git");
        assert_eq!(git_config.remote_origin.get("url"), Some(&"git@github.com:example/repo.git".to_string()));
    }

    #[test]
    fn parse_line_valid_branch_main_key_value() {
        let mut git_config = GitConfig::new();
        git_config.parse_line("remote = origin");
        assert_eq!(git_config.branch_main.get("remote"), Some(&"origin".to_string()));
    }

    #[test]
    fn parse_line_invalid_key_value() {
        let mut git_config = GitConfig::new();
        git_config.parse_line("invalid_key = invalid_value");
        assert!(git_config.core.is_empty());
        assert!(git_config.remote_origin.is_empty());
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn parse_line_invalid_syntax() {
        let mut git_config = GitConfig::new();
        git_config.parse_line("invalid_syntax_without_equal_sign");
        assert!(git_config.core.is_empty());
        assert!(git_config.remote_origin.is_empty());
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn test_read_git_config() {
        // Crea un archivo de prueba temporal con contenido de configuración Git
        let temp_file_path = "test_git_config.txt";
        let mut temp_file = std::fs::File::create(temp_file_path).expect("Failed to create temp file");
        writeln!(temp_file, "repositoryformatversion = 0").expect("Failed to write to temp file");
        writeln!(temp_file, "[remote \"origin\"]").expect("Failed to write to temp file");
        writeln!(temp_file, "   url = git@github.com:example/repo.git").expect("Failed to write to temp file");
        writeln!(temp_file, "[branch \"main\"]").expect("Failed to write to temp file");
        writeln!(temp_file, "   remote = origin").expect("Failed to write to temp file");

        // Crea una instancia de GitConfig y lee la configuración desde el archivo temporal
        let mut git_config = GitConfig::new();
        let result = git_config.read_git_config(temp_file_path);

        // Verifica que la lectura sea exitosa y que las secciones se hayan analizado correctamente
        assert!(result.is_ok());
        assert_eq!(git_config.core["repositoryformatversion"], "0");
        assert_eq!(git_config.remote_origin["url"], "git@github.com:example/repo.git");
        assert_eq!(git_config.branch_main["remote"], "origin");

        // Limpia el archivo temporal después de las pruebas
        std::fs::remove_file(temp_file_path).expect("Failed to remove temp file");
    }

    #[test]
    fn test_read_git_config_nonexistent_file() {
        // Crea una instancia de GitConfig y trata de leer desde un archivo inexistente
        let mut git_config = GitConfig::new();
        let result = git_config.read_git_config("nonexistent_file.txt");

        // Verifica que la lectura no sea exitosa y no se haya modificado la configuración
        assert!(result.is_ok());
        assert!(git_config.core.is_empty());
        assert!(git_config.remote_origin.is_empty());
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn test_add_entry_core() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("repositoryformatversion", "0", "core");

        assert_eq!(git_config.core["repositoryformatversion"], "0");
        assert!(git_config.remote_origin.is_empty());
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn test_add_entry_remote_origin() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");

        assert!(git_config.core.is_empty());
        assert_eq!(git_config.remote_origin["url"], "git@github.com:example/repo.git");
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn test_add_entry_branch_main() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("remote", "origin", "branch.main");

        assert!(git_config.core.is_empty());
        assert!(git_config.remote_origin.is_empty());
        assert_eq!(git_config.branch_main["remote"], "origin");
    }

    #[test]
    fn test_add_entry_unknown_section() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("key", "value", "unknown_section");

        assert!(git_config.core.is_empty());
        assert!(git_config.remote_origin.is_empty());
        assert!(git_config.branch_main.is_empty());
    }

    #[test]
    fn test_get_value_core() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("repositoryformatversion", "0", "core");

        assert_eq!(git_config.get_value("repositoryformatversion", "core"), Some(&"0".to_string()));
        assert_eq!(git_config.get_value("filemode", "core"), None);
    }

    #[test]
    fn test_get_value_remote_origin() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");

        assert_eq!(
            git_config.get_value("url", "remote.origin"),
            Some(&"git@github.com:example/repo.git".to_string())
        );
        assert_eq!(git_config.get_value("fetch", "remote.origin"), None);
    }

    #[test]
    fn test_get_value_branch_main() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("remote", "origin", "branch.main");

        assert_eq!(git_config.get_value("remote", "branch.main"), Some(&"origin".to_string()));
        assert_eq!(git_config.get_value("merge", "branch.main"), None);
    }

    #[test]
    fn test_get_value_unknown_section() {
        let git_config = GitConfig::new();
        assert_eq!(git_config.get_value("key", "unknown_section"), None);
    }

    #[test]
    fn test_write_to_file() {
        let mut git_config = GitConfig::new();
        git_config.add_entry("repositoryformatversion", "0", "core");
        git_config.add_entry("url", "git@github.com:example/repo.git", "remote.origin");
        git_config.add_entry("remote", "origin", "branch.main");

        let file_path = "test_git_config.txt";
        let _ = git_config.write_to_file(file_path);

        let mut file_content = String::new();
        let mut file = File::open(file_path).expect("Could not open file");
        file.read_to_string(&mut file_content).expect("Could not read file");

        let expected_content = "repositoryformatversion = 0\n\
                                [remote \"origin\"]\n    \
                                url = git@github.com:example/repo.git\n\
                                [branch \"main\"]\n    \
                                remote = origin\n";

        assert_eq!(file_content, expected_content);

        // Cleanup
        fs::remove_file(file_path).expect("Could not remove file");
    }

}
