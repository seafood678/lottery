use crate::models::{AppState, Person, Prize, Winner};
use crate::storage::{export_winners_to_json, export_winners_to_text, load_state, save_state};
use rand::seq::SliceRandom;
use std::sync::Mutex;
use tauri::State;

pub struct AppStateWrapper {
    pub state: Mutex<AppState>,
}

#[tauri::command]
pub fn add_person(
    name: String,
    department: Option<String>,
    employee_id: Option<String>,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Person, String> {
    let person = Person::new(name, department, employee_id);
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    app_state.persons.push(person.clone());
    save_state(app.config().as_ref(), &app_state)?;
    Ok(person)
}

#[tauri::command]
pub fn add_persons_batch(
    persons_data: Vec<(String, Option<String>, Option<String>)>,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Vec<Person>, String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    let mut added_persons = Vec::new();

    for (name, department, employee_id) in persons_data {
        let person = Person::new(name, department, employee_id);
        added_persons.push(person.clone());
        app_state.persons.push(person);
    }

    save_state(app.config().as_ref(), &app_state)?;
    Ok(added_persons)
}

#[tauri::command]
pub fn remove_person(
    id: String,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    app_state.persons.retain(|p| p.id != id);
    save_state(app.config().as_ref(), &app_state)?;
    Ok(())
}

#[tauri::command]
pub fn get_all_persons(state: State<AppStateWrapper>) -> Result<Vec<Person>, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.persons.clone())
}

#[tauri::command]
pub fn get_available_persons(state: State<AppStateWrapper>) -> Result<Vec<Person>, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    let winner_ids: Vec<&str> = app_state.winners.iter().map(|w| w.person.id.as_str()).collect();
    let available: Vec<Person> = app_state
        .persons
        .iter()
        .filter(|p| !winner_ids.contains(&p.id.as_str()))
        .cloned()
        .collect();
    Ok(available)
}

#[tauri::command]
pub fn add_prize(
    name: String,
    description: String,
    count: u32,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Prize, String> {
    let prize = Prize::new(name, description, count);
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    app_state.prizes.push(prize.clone());
    save_state(app.config().as_ref(), &app_state)?;
    Ok(prize)
}

#[tauri::command]
pub fn update_prize(
    id: String,
    name: String,
    description: String,
    count: u32,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Prize, String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    let prize = app_state
        .prizes
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("奖项不存在")?;

    let used_count = prize.total_count - prize.remaining_count;
    if count < used_count {
        return Err(format!("新数量不能少于已抽出的数量 ({})", used_count));
    }

    prize.name = name;
    prize.description = description;
    prize.remaining_count = count - used_count;
    prize.total_count = count;

    let updated_prize = prize.clone();
    save_state(app.config().as_ref(), &app_state)?;
    Ok(updated_prize)
}

#[tauri::command]
pub fn remove_prize(
    id: String,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;

    let has_winners = app_state.winners.iter().any(|w| w.prize_id == id);
    if has_winners {
        return Err("该奖项已有中奖者，无法删除".to_string());
    }

    app_state.prizes.retain(|p| p.id != id);
    save_state(app.config().as_ref(), &app_state)?;
    Ok(())
}

#[tauri::command]
pub fn get_all_prizes(state: State<AppStateWrapper>) -> Result<Vec<Prize>, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.prizes.clone())
}

#[tauri::command]
pub fn draw_lottery(
    prize_id: String,
    count: u32,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<Vec<Winner>, String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;

    let prize = app_state
        .prizes
        .iter_mut()
        .find(|p| p.id == prize_id)
        .ok_or("奖项不存在")?;

    if prize.remaining_count < count {
        return Err(format!(
            "奖项剩余数量不足 (剩余: {}, 需要: {})",
            prize.remaining_count, count
        ));
    }

    let prize_name = prize.name.clone();
    let prize_id_clone = prize.id.clone();

    let winner_ids: Vec<String> = app_state.winners.iter().map(|w| w.person.id.clone()).collect();
    let mut available: Vec<Person> = app_state
        .persons
        .iter()
        .filter(|p| !winner_ids.contains(&p.id))
        .cloned()
        .collect();

    if available.len() < count as usize {
        return Err(format!(
            "可抽奖人数不足 (可用: {}, 需要: {})",
            available.len(),
            count
        ));
    }

    let mut rng = rand::thread_rng();
    available.shuffle(&mut rng);

    let selected: Vec<Person> = available.into_iter().take(count as usize).collect();
    let mut new_winners = Vec::new();

    for person in selected {
        let winner = Winner::new(person, prize_id_clone.clone(), prize_name.clone());
        new_winners.push(winner);
    }

    let prize = app_state.prizes.iter_mut().find(|p| p.id == prize_id_clone).unwrap();
    prize.remaining_count -= count;

    app_state.winners.extend(new_winners.clone());
    save_state(app.config().as_ref(), &app_state)?;

    Ok(new_winners)
}

#[tauri::command]
pub fn revoke_winner(
    winner_id: String,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;

    let winner = app_state
        .winners
        .iter()
        .find(|w| w.id == winner_id)
        .ok_or("中奖记录不存在")?
        .clone();

    if let Some(prize) = app_state.prizes.iter_mut().find(|p| p.id == winner.prize_id) {
        prize.remaining_count += 1;
    }

    app_state.winners.retain(|w| w.id != winner_id);
    save_state(app.config().as_ref(), &app_state)?;

    Ok(())
}

#[tauri::command]
pub fn get_all_winners(state: State<AppStateWrapper>) -> Result<Vec<Winner>, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.winners.clone())
}

#[tauri::command]
pub fn export_winners_json(state: State<AppStateWrapper>) -> Result<String, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    export_winners_to_json(&app_state)
}

#[tauri::command]
pub fn export_winners_text(state: State<AppStateWrapper>) -> Result<String, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(export_winners_to_text(&app_state))
}

#[tauri::command]
pub fn save_data(state: State<AppStateWrapper>, app: tauri::AppHandle) -> Result<(), String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    save_state(app.config().as_ref(), &app_state)
}

#[tauri::command]
pub fn load_data(state: State<AppStateWrapper>, app: tauri::AppHandle) -> Result<AppState, String> {
    let loaded = load_state(app.config().as_ref())?;
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    *app_state = loaded.clone();
    Ok(loaded)
}

#[tauri::command]
pub fn reset_all_data(state: State<AppStateWrapper>, app: tauri::AppHandle) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    *app_state = AppState::default();
    save_state(app.config().as_ref(), &app_state)?;
    Ok(())
}

#[tauri::command]
pub fn clear_winners(state: State<AppStateWrapper>, app: tauri::AppHandle) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;

    for prize in &mut app_state.prizes {
        prize.remaining_count = prize.total_count;
    }

    app_state.winners.clear();
    save_state(app.config().as_ref(), &app_state)?;
    Ok(())
}

#[tauri::command]
pub fn get_app_title(state: State<AppStateWrapper>) -> Result<String, String> {
    let app_state = state.state.lock().map_err(|e| e.to_string())?;
    Ok(app_state.app_title.clone())
}

#[tauri::command]
pub fn set_app_title(
    title: String,
    state: State<AppStateWrapper>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut app_state = state.state.lock().map_err(|e| e.to_string())?;
    app_state.app_title = title;
    save_state(app.config().as_ref(), &app_state)?;
    Ok(())
}
