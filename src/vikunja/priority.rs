pub fn map_priority(priority: &str) -> i64 {
    match priority.to_uppercase().as_str() {
        "NONE" => 0,
        "LOW" => 1,
        "MEDIUM" => 2,
        "HIGH" => 3,
        "URGENT" => 4,
        "DO_NOW" | "CRITICAL" => 5,
        _ => 0, // Fallback to None
    }
}