use config::Config;
use config::builders::Builder as B;

use super::user;


pub fn build_container(config: &Config, name: &String) -> Result<(), String> {
    let container = try!(config.containers.get(name)
        .ok_or(format!("Container {:?} not found", name)));
    for i in container.setup.iter() {
        if let &B::Container(ref name) = i {
            try!(build_container(config, name));
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
