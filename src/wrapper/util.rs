use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use config::Container;


pub fn find_cmd(cmd: &str, env: &BTreeMap<String, String>)
    -> Result<PathBuf, String>
{
    if cmd.contains("/") {
        return Ok(PathBuf::from(cmd));
    } else {
        if let Some(paths) = env.get(&"PATH".to_string()) {
            for dir in paths[..].split(':') {
                let path = Path::new(dir);
                if !path.is_absolute() {
                    warn!("All items in PATH must be absolute, not {:?}",
                          path);
                    continue;
                }
                let path = path.join(cmd);
                if path.exists() {
                    return Ok(path);
                }
            }
            return Err(format!("Command {} not found in {:?}",
                cmd, paths));
        } else {
            return Err(format!("Command {} is not absolute and no PATH set",
                cmd));
        }
    }
}

pub fn check_data_container(container_config: &Container) {
    if container_config.is_data_container() {
        warn!("You are trying to run command inside the data container. \
            Data containers is designed to use as volumes inside other \
            containers. Usually there are no system dirs at all.");
    }
}
