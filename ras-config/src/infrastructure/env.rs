use std::path::Path;

pub fn load_env() {
    let _ = dotenvy::dotenv();
}

pub fn load_env_from(path: &Path) {
    let _ = dotenvy::from_path(path);
}
