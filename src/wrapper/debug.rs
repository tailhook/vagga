use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Stdio};

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
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit()).stderr(Stdio::inherit())
            .arg("sh")
            .env("PATH", "/vagga/bin")
        .output()
        .map_err(|e| format!("Can't run tar: {}", e))
        .map(|o| o.status)
    {
        Ok(x) if x.signal().is_some() => 128+(x.signal().unwrap() as i32),
        Ok(x) => x.code().unwrap() as i32,
        Err(x) => {
            error!("Error running build_shell: {}", x);
            return 127;
        }
    }
}
