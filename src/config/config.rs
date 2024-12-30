use std::fs;

pub struct Config {
    pub analyst_config_path: String,
}

// load config from toml file
pub fn load_config(file_path: &str) -> String {
    let contents = fs::read_to_string(file_path)
        .expect("Something went wrong reading the file");
    if contents.is_empty() {
        println!("File is empty");
    } else {
        println!("File is not empty");
    }
    contents
}
