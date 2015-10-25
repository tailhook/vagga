use std::path::Path;

use unshare::{Command, Stdio};

use container::uidmap::{map_users};
use options::pack::Options;
use wrapper::Wrapper;
use process_util::{convert_status, set_uidmap};
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
    let root = Path::new("/vagga/base/.roots")
        .join(container_ver).join("root");

    let mut cmd = Command::new("/vagga/bin/busybox");
    cmd.stdin(Stdio::null())
        .arg("tar")
        .arg("-c");
    set_uidmap(&mut cmd,
        &try!(map_users(wrapper.settings, &cconfig.uids, &cconfig.gids)),
        false);
    if let Some(ref f) = options.file {
        cmd.arg("-f").arg(Path::new("/work").join(f));
    }
    cmd.arg("-C").arg(&root);
    cmd.arg(".");

    info!("Running {:?}", cmd);
    cmd.status()
        .map(convert_status)
        .map_err(|e| format!("Command {:?} {}", cmd, e))
}
