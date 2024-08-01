use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};

use crate::servers::errors::ServerError;



pub fn save_pr_map(pr_map_path: &str, pr_map: &HashMap<String, u64>) -> Result<(), ServerError> {
    let file_content = serde_json::to_string_pretty(pr_map).map_err(|_| ServerError::SaveMapPrFile)?;
    std::fs::write(pr_map_path, file_content).map_err(|_| ServerError::SaveMapPrFile)?;
    Ok(())
}

pub fn generate_head_base_hash(head: &str, base: &str) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{}:{}", head, base).hash(&mut hasher);
    let hash = hasher.finish();
    format!("{:x}", hash)
}

pub fn read_pr_map(pr_map_path: &str) -> Result<HashMap<String, u64>, ServerError> {
    let file_content = std::fs::read_to_string(pr_map_path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&file_content).map_err(|_| ServerError::ReadMapPrFile)
}