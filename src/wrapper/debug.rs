use unshare::{Command, Stdio};

use process_util::convert_status;
use super::Wrapper;
use super::setup::setup_base_filesystem;


pub fn run_interactive_build_shell(wrapper: &Wrapper) -> i32 {
    if let Err(text) = setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)
    {
        error!("Error setting base file system: {}", &text);
        return 122;
    }
    match Command::new("/vagga/bin/busybox")
            .arg("sh")
            .env("PATH", "/vagga/bin")
        .status()
        .map_err(|e| format!("Can't run busybox: {}", e))
    {
        Ok(x) => convert_status(x),
        Err(x) => {
            error!("Error running build_shell: {}", x);
            return 127;
        }
    }
}
