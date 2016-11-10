use std::default::Default;
use std::io::{stderr, Write};
use std::path::Path;

use unshare::{Command, Stdio};

use builder::capsule;
use options::pack::Options;
use wrapper::Wrapper;
use process_util::convert_status;
use super::setup;


pub fn pack_image_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let options = match Options::parse(&cmdline) {
        Ok(x) => x,
        Err(code) => return Ok(code),
    };

    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    let mut capsule_features = vec!();
    let container_ver = wrapper.root.as_ref().unwrap();
    let root = Path::new("/vagga/base/.roots")
        .join(container_ver)
        .join("root");
    let mut tar_cmd = Command::new("/vagga/bin/busybox");
    tar_cmd.stdin(Stdio::null())
        .arg("tar")
        .arg("-c");
    if let Some(compression_type) = options.compression_type {
        tar_cmd.arg(compression_type.get_short_option());
        capsule_features.push(compression_type.get_capsule_feature());
    }
    if let Some(ref f) = options.file {
        tar_cmd.arg("-f");
        if f.starts_with("/") {
            tar_cmd.arg(f);
        } else {
            tar_cmd.arg(Path::new("/work").join(f));
        }
    }
    tar_cmd
        .arg("-C").arg(&root)
        .arg(".");

    if capsule_features.len() > 0 {
        let mut capsule_state = Default::default();
        capsule::ensure(
            &mut capsule_state, &wrapper.settings, &capsule_features)?;
    }

    if let Some(_) = options.compression_type {
        writeln!(&mut stderr(),
            "Compressing the image... This may take a few minutes.").ok();
    }
    info!("Running {:?}", tar_cmd);
    tar_cmd.status()
        .map(convert_status)
        .map_err(|e| format!("Error running {:?}: {}", tar_cmd, e))
}
