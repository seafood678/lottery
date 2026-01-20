use crate::models::AppState;
use std::fs;
use std::path::PathBuf;
use tauri::api::path::app_data_dir;
use tauri::Config;

const DATA_FILE: &str = "lottery_data.json";

pub fn get_data_path(config: &Config) -> PathBuf {
    let app_dir = app_data_dir(config).unwrap_or_else(|| PathBuf::from("."));
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).ok();
    }
    app_dir.join(DATA_FILE)
}

pub fn save_state(config: &Config, state: &AppState) -> Result<(), String> {
    let path = get_data_path(config);
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("序列化失败: {}", e))?;
    fs::write(&path, json)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    Ok(())
}

pub fn load_state(config: &Config) -> Result<AppState, String> {
    let path = get_data_path(config);
    if !path.exists() {
        return Ok(AppState::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    let state: AppState = serde_json::from_str(&content)
        .map_err(|e| format!("解析数据失败: {}", e))?;
    Ok(state)
}

pub fn export_winners_to_json(state: &AppState) -> Result<String, String> {
    serde_json::to_string_pretty(&state.winners)
        .map_err(|e| format!("导出失败: {}", e))
}

pub fn export_winners_to_text(state: &AppState) -> String {
    let mut result = String::from("========== 中奖名单 ==========\n\n");

    let mut prize_winners: std::collections::HashMap<String, Vec<&crate::models::Winner>> =
        std::collections::HashMap::new();

    for winner in &state.winners {
        prize_winners
            .entry(winner.prize_name.clone())
            .or_default()
            .push(winner);
    }

    for (prize_name, winners) in prize_winners {
        result.push_str(&format!("【{}】\n", prize_name));
        for (i, winner) in winners.iter().enumerate() {
            let dept = winner.person.department.as_deref().unwrap_or("无部门");
            result.push_str(&format!("  {}. {} ({})\n", i + 1, winner.person.name, dept));
        }
        result.push('\n');
    }

    result
}
