use std::{collections::HashMap, fs};
pub fn get_app_settings(args: Vec<String>) -> HashMap<String, String> {
    let mut settings = HashMap::new();
    let mut found = false;
    for i in 0..args.len() {
        if args[i].starts_with("--") {
            found = true;
        } else if found {
            settings.insert(args[i - 1].clone(), args[i].clone());
            found = false;
        }
    }

    settings
}

pub fn get_code_from_file(path: String) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn get_i64_from_settings(settings: &HashMap<String, String>, key: &String) -> i64 {
    settings.get(key).unwrap().parse().unwrap()
}

pub fn get_usize_from_settings(
    settings: &HashMap<String, String>,
    key: String,
    default: String,
) -> usize {
    settings.get(&key).unwrap_or(&default).parse().unwrap()
}
pub fn get_string_from_settings(
    settings: &HashMap<String, String>,
    key: String,
    default: String,
) -> String {
    settings.get(&key).unwrap_or(&default).parse().unwrap()
}
