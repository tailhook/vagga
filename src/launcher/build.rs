use config::Config;

use super::user;


pub fn build_container(config: &Config, name: &String) -> Result<(), String> {
    match user::run_wrapper(None, "_build".to_string(),
                            vec!(name.to_string()), true)
    {
        Ok(0) => Ok(()),
        Ok(x) => Err(format!("Build returned {}", x)),
        Err(e) => Err(e),
    }
}
