use std::env;
use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};
use unshare::{Command, Namespace};

use config::Config;
use process_util::{capture_fd3, set_uidmap};
use container::uidmap::get_max_uidmap;


pub fn build_container(_config: &Config, name: &String)
    -> Result<String, String>
{
    build_internal(name, &[])
}

/// Similar to build_container but never actually builds
pub fn get_version(name: &str) -> Result<String, String> {
    let mut cmd = Command::new("/proc/self/exe");
    cmd.arg0("vagga_wrapper");
    cmd.arg("_version_hash");
    cmd.arg("--short");
    cmd.arg("--fd3");
    cmd.arg(name);
    cmd.env_clear();
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("VAGGA_USER_HOME", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

fn build_internal(name: &str, args: &[String]) -> Result<String, String> {
    let mut cmd = Command::new("/proc/self/exe");
    cmd.arg0("vagga_wrapper");
    cmd.arg("_build");
    cmd.arg(name);
    cmd.args(&args);
    cmd.env_clear();
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("VAGGA_USER_HOME", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

pub fn build_command(_config: &Config, mut args: Vec<String>)
    -> Result<i32, String>
{
    let mut name: String = "".to_string();
    let mut force: bool = false;
    {
        let mut cmdline = args.clone();
        cmdline.insert(0, "vagga _build".to_string());
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Internal vagga tool to setup basic system sandbox
            ");
        ap.refer(&mut name)
            .add_argument("container_name", Store,
                "Container name to build");
        ap.refer(&mut force)
            .add_option(&["--force"], StoreTrue,
                "Force build even if container is considered up to date");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    assert!(args.remove(0) == name);

    build_internal(&name, &args)
    .map(|v| debug!("Container {:?} build with version {:?}", name, v))
    .map(|()| 0)
}
