use config::Config;
use config::builders::Builder as B;

use super::user;


pub fn build_container(config: &Config, name: &String) -> Result<(), String> {
    let container = try!(config.containers.get(name)
        .ok_or(format!("Container {:?} not found", name)));
    for step in container.setup.iter() {
        match step {
            &B::Container(ref name) => {
                try!(build_container(config, name));
            }
            &B::SubConfig(ref cfg) => {
                if let Some(ref name) = cfg.generator {
                    try!(build_container(config, name));
                }
            }
            _ => {}
        }
    }
    match user::run_wrapper(None, "_build".to_string(),
                            vec!(name.to_string()), true)
    {
        Ok(0) => Ok(()),
        Ok(x) => Err(format!("Build returned {}", x)),
        Err(e) => Err(e),
    }
}
