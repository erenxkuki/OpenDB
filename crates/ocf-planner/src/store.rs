use crate::plan::CompressionPlan;
use std::path::Path;

pub fn save_plan(plan: &CompressionPlan, path: &Path) -> Result<(), std::io::Error> {
    let json = serde_json::to_string_pretty(plan).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

pub fn load_plan(path: &Path) -> Result<CompressionPlan, std::io::Error> {
    let json = std::fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
