use std::process::Command;

use super::Wrapper;
use super::setup::setup_base_filesystem;


pub fn run_interactive_build_shell(wrapper: &Wrapper) -> i32 {
    if let Err(text) = setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)
    {
        error!("Error setting base file system: {}", text.as_slice());
        return 122;
    }
    match Command::new("/vagga/bin/busybox")
            .stdin(InheritFd(0)).stdout(InheritFd(1)).stderr(InheritFd(2))
            .arg("sh")
            .env("PATH", "/vagga/bin")
        .output()
        .map_err(|e| format!("Can't run tar: {}", e))
        .map(|o| o.status)
    {
        Ok(ExitStatus(x)) => x as i32,
        Ok(ExitSignal(x)) => 128+(x as i32),
        Err(x) => {
            error!("Error running build_shell: {}", x);
            return 127;
        }
    }
}
