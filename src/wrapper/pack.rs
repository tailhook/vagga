use std::env;
use std::path::{Path, PathBuf};

use rustc_serialize::json;

use unshare::{Command, Namespace, Stdio};

use container::uidmap::{map_users};
use options::pack::Options;
use wrapper::Wrapper;
use process_util::{convert_status, copy_env_vars, set_uidmap};
use super::build;
use super::setup;


pub fn pack_image_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let options = match Options::parse(&cmdline) {
        Ok(x) => x,
        Err(code) => return Ok(code),
    };

    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));
    let cconfig = try!(wrapper.config.containers.get(&options.name)
        .ok_or(format!("Container {} not found", options.name)));
    let container_ver = wrapper.root.as_ref().unwrap();

    let uid_map = try!(map_users(wrapper.settings,
                                 &cconfig.uids, &cconfig.gids));

    let tmppath = PathBuf::from(
        &format!("/vagga/base/.roots/.tmp.{}", &options.name));
    match build::prepare_tmp_root_dir(&tmppath) {
        Ok(()) => {}
        Err(x) => {
            return Err(format!("Error preparing root dir: {}", x));
        }
    }

    // we run internal vagga_push command cause we need capsule
    let mut cmd = Command::new("/vagga/bin/vagga");
    cmd.stdin(Stdio::null());
    cmd.arg0("vagga_pack");
    set_uidmap(&mut cmd, &uid_map, false);
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    cmd.arg(&options.name);
    if let Some(ref f) = options.file {
        cmd.arg("-f");
        if f.starts_with("/") {
            cmd.arg(f);
        } else {
            cmd.arg(Path::new("/work").join(f));
        }
            
    }
    if let Some(ref compression_type) = options.compression_type {
        cmd.arg("-t");
        cmd.arg(compression_type);
    }
    cmd.arg("--container-version");
    cmd.arg(&container_ver);
    cmd.arg("--settings");
    cmd.arg(json::encode(wrapper.settings).unwrap());
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    let result = cmd.status();
    result
        .map(convert_status)
        .map_err(|e| format!("Command {:?} {}", cmd, e))
}
