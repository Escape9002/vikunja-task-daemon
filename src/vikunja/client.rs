use anyhow::{anyhow, Result};
use reqwest::Client;
use super::models::{CreateTaskRequest, Paginated, Project, Task};

pub struct VikunjaClient {
    client: Client,
    base_url: String,
    token: String,
}

impl VikunjaClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        let url = format!("{}/projects", self.base_url);
        let res = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Failed to fetch projects: {}", res.status()));
        }
        
        // v2: Extract array from the "items" field of the pagination envelope
        let paginated: Paginated<Project> = res.json().await?;
        Ok(paginated.items)
    }

    pub async fn get_tasks(&self, project_id: i64) -> Result<Vec<Task>> {
        let url = format!("{}/projects/{}/tasks?per_page=100", self.base_url, project_id);
        let res = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Failed to fetch tasks: {}", res.status()));
        }
        
        // v2: Extract array from the "items" field of the pagination envelope
        let paginated: Paginated<Task> = res.json().await?;
        Ok(paginated.items)
    }

    pub async fn create_task(
        &self,
        project_id: i64,
        title: String,
        due_date: String,
        priority: i64,
    ) -> Result<()> {
        let url = format!("{}/projects/{}/tasks", self.base_url, project_id);
        let body = CreateTaskRequest {
            title,
            due_date,
            priority,
        };

        let res = self
            .client
            .post(&url) // v2: Create is now POST (returns 201 Created)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Failed to create task: {}", res.status()));
        }
        Ok(())
    }

    pub async fn move_task(&self, task_id: i64, new_project_id: i64) -> Result<()> {
        let url = format!("{}/tasks/{}", self.base_url, task_id);
        
        // v2: Partial update using JSON Merge Patch
        let body = serde_json::json!({
            "project_id": new_project_id
        });

        let res = self
            .client
            .patch(&url) // v2: Patch used for partial updates
            .bearer_auth(&self.token)
            .json(&body) // Reqwest automatically sets Content-Type: application/json
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Failed to move task: {}", res.status()));
        }
        
        Ok(())
    }
}