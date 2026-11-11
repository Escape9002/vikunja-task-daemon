use serde::{Deserialize, Serialize};

// New v2 envelope wrapper for list endpoints
#[derive(Debug, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: i64,
    pub title: String,
    pub parent_project_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub project_id: i64,
    pub due_date: Option<String>, 
}

#[derive(Debug, Serialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub due_date: String,
    pub priority: i64,
}