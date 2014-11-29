use std::os::getcwd;
use std::str::from_utf8;
use std::io::process::{Command, ExitStatus};

mod symlink_mnt { mod run; }
mod symlink_vagga { mod run; }
mod empty_container { mod run; }


pub fn vagga_cmd() -> Command {
    let path = getcwd().join("vagga");
    return Command::new(path);
}

pub fn check_status_output(cmd: Command, status: int,
                           stdout: &str, stderr: &str)
{
    let out = cmd.spawn().unwrap().wait_with_output().unwrap();
    let mut err = false;
    if out.status != ExitStatus(status) {
        println!("Command {} returned {} instead {}", cmd, out.status, status);
        err = true;
    }
    if out.output.as_slice() != stdout.as_bytes().as_slice() {
        println!("Command {} errorneous stdout:\n{}", cmd,
            from_utf8(out.output.as_slice()));
        err = true;
    }
    if out.error.as_slice() != stderr.as_bytes().as_slice() {
        println!("Command {} errorneous stderr:\n{}", cmd,
            from_utf8(out.error.as_slice()));
        err = true;
    }
    if err {
        fail!();
    }
}

