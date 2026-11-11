use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub tasks: HashMap<String, HashMap<String, TaskSpec>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskSpec {
    #[serde(default = "default_backlog_name")]
    pub backlog_project_name: String,
    pub repetition: String,
    pub due_day: String,
    pub due_time: String,
    pub importance: String,
    pub name_template: String,
    #[serde(default = "default_pre_generation")]
    pub pre_generation: usize,
}

fn default_backlog_name() -> String {
    "Backlog".to_string()
}

fn default_pre_generation() -> usize {
    1
}

impl AppConfig {
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}