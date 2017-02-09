use std::env;
use std::io::{stdout, stderr};

use argparse::{ArgumentParser, Store, StoreTrue};
use unshare::{Command, Namespace};

use launcher::Context;
use launcher::wrap::Wrapper;
use options::build_mode::BuildMode;
use process_util::{capture_fd3, copy_env_vars, squash_stdio};
use container::util::version_from_symlink;


pub fn build_container(context: &Context, name: &String, mode: BuildMode)
    -> Result<String, String>
{
    use options::build_mode::BuildMode::*;
    let ver = match mode {
        Normal => build_internal(context, name, &[])?,
        NoImage => build_internal(context, name,
            &["--no-image-download".to_string()])?,
        NoBuild => format!("{}.{}", &name, get_version(context, &name)?),
        NoVersion => {
            version_from_symlink(format!(".vagga/{}", name))?
        }
    };
    Ok(ver)
}

/// Similar to build_container but never actually builds
pub fn get_version(context: &Context, name: &str) -> Result<String, String> {
    let mut cmd: Command = Wrapper::new(None, &context.settings);
    cmd.arg("_version_hash");
    cmd.arg("--short");
    cmd.arg("--fd3");
    cmd.arg(name);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &context.settings);
    // TODO(tailhook) move these to copy_env_vars, or at least reuse in build?
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("_VAGGA_HOME", x);
    }
    if let Some(x) = env::var_os("VAGGA_SETTINGS") {
        cmd.env("VAGGA_SETTINGS", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    cmd.map_users_for(context.config.get_container(name)?, &context.settings)?;

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

fn build_internal(context: &Context, name: &str, args: &[String])
    -> Result<String, String>
{
    let mut cmd = Wrapper::new(None, &context.settings);
    squash_stdio(&mut cmd)?;
    cmd.arg("_build");
    cmd.arg(name);
    cmd.args(&args);
    cmd.env_clear();
    copy_env_vars(&mut cmd, &context.settings);
    // TODO(tailhook) move these to copy_env_vars, or at least
    // reuse in ver and capsule?
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    if let Ok(x) = env::var("HOME") {
        cmd.env("_VAGGA_HOME", x);
    }
    if let Some(x) = env::var_os("VAGGA_SETTINGS") {
        cmd.env("VAGGA_SETTINGS", x);
    }
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    cmd.map_users_for(context.config.get_container(name)?, &context.settings)?;

    capture_fd3(cmd)
    .and_then(|x| String::from_utf8(x)
                  .map_err(|e| format!("Can't decode version: {}", e)))
}

pub fn build_command(context: &Context, args: Vec<String>)
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
    if let BuildMode::NoImage = context.build_mode {
        args.push("--no-image-download".to_string());
    }

    build_internal(context, &name, &args)
    .map(|v| debug!("Container {:?} build with version {:?}", name, v))
    .map(|()| 0)
}
