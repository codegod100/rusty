use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct RandomFact {
    text: String,
    #[serde(rename = "type")]
    fact_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CatFact {
    fact: String,
    length: u32,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_random_data() -> Result<String, String> {
    // Try to get a random cat fact
    match ureq::get("https://catfact.ninja/fact").call() {
        Ok(response) => {
            match response.into_json::<CatFact>() {
                Ok(cat_fact) => Ok(format!("🐱 Cat Fact: {}", cat_fact.fact)),
                Err(_) => {
                    // Fallback to a different API
                    get_random_advice()
                }
            }
        }
        Err(_) => {
            // Fallback to a different API
            get_random_advice()
        }
    }
}

fn get_random_advice() -> Result<String, String> {
    #[derive(Deserialize)]
    struct AdviceResponse {
        slip: AdviceSlip,
    }
    
    #[derive(Deserialize)]
    struct AdviceSlip {
        advice: String,
    }
    
    match ureq::get("https://api.adviceslip.com/advice").call() {
        Ok(response) => {
            match response.into_json::<AdviceResponse>() {
                Ok(advice_response) => Ok(format!("💡 Random Advice: {}", advice_response.slip.advice)),
                Err(e) => Err(format!("Failed to parse advice: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to fetch advice: {}", e)),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_random_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
