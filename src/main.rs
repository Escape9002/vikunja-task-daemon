mod config;
mod scheduler;
mod vikunja;
mod watcher;

use anyhow::Result;
use regex::Regex;
use std::env;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    info!("Starting vikunja-task-daemon...");

    let api_url = env::var("VIKUNJA_API_URL").expect("VIKUNJA_API_URL must be set");
    let api_token = env::var("VIKUNJA_API_TOKEN").expect("VIKUNJA_API_TOKEN must be set");
    let config_path =
        env::var("CONFIG_FILE_PATH").unwrap_or_else(|_| "/etc/vikunja-daemon/config.yaml".into());
    let interval_secs: u64 = env::var("CHECK_INTERVAL_SECS")
        .unwrap_or_else(|_| "60".to_string())
        .parse()
        .unwrap_or(60);
    let timezone = env::var("TIMEZONE").unwrap_or_else(|_| "UTC".to_string());

    let initial_config = config::AppConfig::load(&config_path)
        .expect("Failed to load initial configuration file");
    
    let config_lock = Arc::new(RwLock::new(initial_config));

    watcher::start_watcher(config_path.clone(), Arc::clone(&config_lock));

    let client = vikunja::client::VikunjaClient::new(api_url, api_token);
    let tz: chrono_tz::Tz = timezone.parse().expect("Invalid TIMEZONE provided");

    info!("Startup ended, entering synchronization loop");
    
    loop {
        debug!("Waking up for evaluation cycle...");
        let current_config = config_lock.read().unwrap().clone();
        let now = chrono::Utc::now().with_timezone(&tz);

        match client.get_projects().await {
            Ok(projects) => {
                for (project_name, tasks_map) in current_config.tasks {
                    let Some(proj) = projects.iter().find(|p| p.title == project_name) else {
                        warn!("Project '{}' not found in Vikunja, skipping.", project_name);
                        continue;
                    };

                    let proj_tasks = client.get_tasks(proj.id).await.unwrap_or_default();

                    for (task_key, task_spec) in tasks_map {
                        let pattern = scheduler::generate_regex_pattern(&task_spec.name_template);
                        let re = Regex::new(&pattern).expect("Invalid regex generated");

                        // 1. Gather and Sort History
                        let mut history: Vec<&vikunja::models::Task> = proj_tasks
                            .iter()
                            .filter(|t| re.is_match(&t.title))
                            .collect();

                        history.sort_by(|a, b| {
                            let date_a = a.due_date.as_deref().unwrap_or("0001-01-01T00:00:00Z");
                            let date_b = b.due_date.as_deref().unwrap_or("0001-01-01T00:00:00Z");
                            date_b.cmp(date_a) // Descending: latest first
                        });

                        // 2. BACKLOG PHASE
                        for old_task in &history {
                            if !old_task.done {
                                if let Some(due_str) = &old_task.due_date {
                                    if let Ok(due_dt) = chrono::DateTime::parse_from_rfc3339(due_str) {
                                        if due_dt.with_timezone(&tz) < now {
                                            // Task is Overdue! Move it.
                                            let backlog_name = &task_spec.backlog_project_name;
                                            if let Some(backlog) = projects.iter().find(|p| {
                                                p.title == *backlog_name && p.parent_project_id == Some(proj.id)
                                            }) {
                                                info!("Moving overdue task '{}' to backlog.", old_task.title);
                                                if let Err(e) = client.move_task(old_task.id, backlog.id).await {
                                                    error!("Failed to move task to backlog: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 3. PIPELINE LIMIT: Count how many future tasks already exist
                        let future_count = history.iter().filter(|t| {
                            if let Some(due_str) = &t.due_date {
                                if let Ok(due_dt) = chrono::DateTime::parse_from_rfc3339(due_str) {
                                    return due_dt.with_timezone(&tz) >= now;
                                }
                            }
                            false
                        }).count();

                        // 4. SCHEDULING PHASE: Generate missing advance tasks
                        if future_count < task_spec.pre_generation {
                            let needed = task_spec.pre_generation - future_count;

                            // Determine start point
                            let mut target_due = if let Some(latest) = history.first() {
                                if let Some(due_str) = &latest.due_date {
                                    let mut d = chrono::DateTime::parse_from_rfc3339(due_str).unwrap().with_timezone(&tz);
                                    if d >= now {
                                        scheduler::add_repetition(&d, &task_spec.repetition).unwrap()
                                    } else {
                                        // User missed many weeks, fast forward the oldest phase until it reaches present
                                        while d < now {
                                            d = scheduler::add_repetition(&d, &task_spec.repetition).unwrap();
                                        }
                                        d
                                    }
                                } else {
                                    scheduler::next_occurrence(&tz, &task_spec.due_day, &task_spec.due_time, now).unwrap()
                                }
                            } else {
                                // Complete initial bootstrap
                                scheduler::next_occurrence(&tz, &task_spec.due_day, &task_spec.due_time, now).unwrap()
                            };

                            for _ in 0..needed {
                                let current_title = scheduler::generate_current_title(&task_spec.name_template, &target_due);

                                // Idempotency check just in case
                                if proj_tasks.iter().any(|t| t.title == current_title) {
                                    debug!("Task '{}' exists internally. Skipping.", current_title);
                                } else {
                                    let target_due_str = target_due.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                                    let priority_int = vikunja::priority::map_priority(&task_spec.importance);

                                    info!("Creating advance task '{}' for {}", current_title, task_key);
                                    if let Err(e) = client.create_task(proj.id, current_title, target_due_str, priority_int).await {
                                        error!("Failed to create task: {}", e);
                                    }
                                }
                                
                                // Step forward to the next future repetition
                                target_due = scheduler::add_repetition(&target_due, &task_spec.repetition).unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Failed to fetch projects from Vikunja: {}", e),
        }

        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}