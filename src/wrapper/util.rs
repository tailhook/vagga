use std::old_io::fs::PathExtensions;
use std::collections::BTreeMap;


pub fn find_cmd(cmd: &str, env: &BTreeMap<String, String>)
    -> Result<Path, String>
{
    if cmd.contains("/") {
        return Ok(Path::new(cmd));
    } else {
        if let Some(paths) = env.get(&"PATH".to_string()) {
            for dir in paths.as_slice().split(':') {
                let path = Path::new(dir);
                if !path.is_absolute() {
                    warn!("All items in PATH must be absolute, not {}",
                          path.display());
                    continue;
                }
                let path = path.join(cmd);
                if path.exists() {
                    return Ok(path);
                }
            }
            return Err(format!("Command {} not found in {}",
                cmd, paths.as_slice()));
        } else {
            return Err(format!("Command {} is not absolute and no PATH set",
                cmd));
        }
    }
}
