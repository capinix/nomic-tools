use crate::globals::PROFILES_DIR;
use eyre::Result;

pub fn save_config() -> Result<()>{
    let file = PROFILES_DIR.join("profile_config.toml");
    println!("{}", "file");
    Ok(())
}

