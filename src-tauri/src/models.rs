use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub department: Option<String>,
    pub employee_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prize {
    pub id: String,
    pub name: String,
    pub description: String,
    pub total_count: u32,
    pub remaining_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Winner {
    pub id: String,
    pub person: Person,
    pub prize_id: String,
    pub prize_name: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub persons: Vec<Person>,
    pub prizes: Vec<Prize>,
    pub winners: Vec<Winner>,
    #[serde(default = "default_app_title")]
    pub app_title: String,
}

fn default_app_title() -> String {
    "年会抽奖系统".to_string()
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            persons: Vec::new(),
            prizes: Vec::new(),
            winners: Vec::new(),
            app_title: default_app_title(),
        }
    }
}

impl Person {
    pub fn new(name: String, department: Option<String>, employee_id: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            department,
            employee_id,
        }
    }
}

impl Prize {
    pub fn new(name: String, description: String, count: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description,
            total_count: count,
            remaining_count: count,
        }
    }
}

impl Winner {
    pub fn new(person: Person, prize_id: String, prize_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            person,
            prize_id,
            prize_name,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}
