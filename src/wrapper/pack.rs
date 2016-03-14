use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};

use builder::capsule;
use builder::context::{Context as BuilderContext};
use options::pack::Options;
use wrapper::Wrapper;
use process_util::convert_status;
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

    let tmppath = PathBuf::from(
        &format!("/vagga/base/.roots/.tmp.{}", &options.name));
    match build::prepare_tmp_root_dir(&tmppath) {
        Ok(()) => {}
        Err(x) => {
            return Err(format!("Error preparing root dir: {}", x));
        }
    }

    let mut capsule_features = vec!();
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
        let mut ctx = BuilderContext::new(&wrapper.config, options.name.clone(),
                                   cconfig, wrapper.settings.clone());
        try!(capsule::ensure_features(&mut ctx, &capsule_features));
    }

    info!("Running {:?}", tar_cmd);
    tar_cmd.status()
        .map(convert_status)
        .map_err(|e| format!("Error running {:?}: {}", tar_cmd, e))
}
