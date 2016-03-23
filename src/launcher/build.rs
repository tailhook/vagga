use std::env;
use std::io::{stdout, stderr};
use std::fs::read_link;
use std::os::unix::io::{FromRawFd, RawFd};

use argparse::{ArgumentParser, Store, StoreTrue};
use nix::unistd::dup;
use unshare::{Command, Namespace, Stdio};

use options::build_mode::BuildMode;
use config::Settings;
use process_util::{capture_fd3, set_uidmap, copy_env_vars, squash_stdio};
use container::uidmap::get_max_uidmap;


pub fn build_container(settings: &Settings, name: &String,
    mode: BuildMode, stdout: Option<RawFd>)
    -> Result<String, String>
{
    use options::build_mode::BuildMode::*;
    let ver = match mode {
        Normal => try!(build_internal(settings, name, &[], stdout)),
        NoImage => try!(build_internal(settings, name,
            &[String::from("--no-image-download")], stdout)),
        NoBuild => format!("{}.{}", &name, try!(get_version(settings, &name))),
        NoVersion => {
            let lnk = format!(".vagga/{}", name);
            let path = try!(read_link(&lnk)
                .map_err(|e| format!("Can't read link {:?}: {}", lnk, e)));
            try!(path.iter().rev().nth(1).and_then(|x| x.to_str())
                .ok_or(format!("Bad symlink {:?}: {:?}", lnk, path)))
                .to_string()
        }
    };
    Ok(ver)
}

/// Similar to build_container but never actually builds
pub fn get_version(settings: &Settings, name: &str) -> Result<String, String> {
    let mut cmd = Command::new("/proc/self/exe");
    cmd.arg0("vagga_wrapper");
    cmd.arg("_version_hash");
    cmd.arg("--short");
    cmd.arg("--fd3");
    cmd.arg(name);
    cmd.env_clear();
    copy_env_vars(&mut cmd, settings);
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("_VAGGA_HOME", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

fn build_internal(settings: &Settings, name: &str, args: &[String], stdout: Option<RawFd>)
    -> Result<String, String>
{
    let mut cmd = Command::new("/proc/self/exe");
    if let Some(stdout_fd) = stdout {
        let fd = try!(dup(stdout_fd).map_err(|e| format!("{}", e)));
        cmd.stdout(unsafe { Stdio::from_raw_fd(fd) });
    }
    cmd.arg0("vagga_wrapper");
    cmd.arg("_build");
    cmd.arg(name);
    cmd.args(&args);
    cmd.env_clear();
    copy_env_vars(&mut cmd, settings);
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("_VAGGA_HOME", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    cmd.make_group_leader(true);
    set_uidmap(&mut cmd, &get_max_uidmap().unwrap(), true);

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

pub fn build_command(settings: &Settings, args: Vec<String>)
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
    let mut args = Vec::new();
    if force {
        args.push("--force".to_string());
    }

    build_internal(settings, &name, &args, None)
    .map(|v| debug!("Container {:?} build with version {:?}", name, v))
    .map(|()| 0)
}
