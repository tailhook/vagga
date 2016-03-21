use std::fs::remove_file;
use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};

use config::read_settings::MergedSettings;
use config::settings::Settings;
use launcher::build::build_container;
use launcher::wrap::Wrapper;
use options::push::Options;


pub fn push_command(ext_settings: &MergedSettings, settings: &Settings, args: Vec<String>)
    -> Result<i32, String>
{
    let mut cmdline = args.clone();
    cmdline.insert(0, "vagga _push_image".to_string());
    let opt = match Options::parse(&cmdline) {
        Ok(x) => x,
        Err(code) => return Ok(code),
    };

    let ver = try!(build_container(settings, &opt.name, opt.build_mode));
    let hash = match ver.rsplitn(2, ".").next() {
        Some(v) => v,
        None => return Err(format!("Incorrect container version")),
    };

    let mut pack_cmd: Command = Wrapper::new(Some(&ver), &settings);
    let image_name = "image.tar.xz";
    let image_path = Path::new("/vagga/base/.roots")
        .join(&ver)
        .join(image_name);
    pack_cmd.userns();
    pack_cmd
        .arg("_pack_image")
        .arg(&opt.name)
        .arg("-f").arg(&image_path)
        .arg("-J");
    match pack_cmd.status() {
        Ok(st) if !st.success() => {
            return Err(format!("Error when packing image: {:?}", pack_cmd));
        },
        Err(e) => {
            return Err(format!("Error when packing image: {}", e));
        },
        _ => {},
    }

    let roots = if ext_settings.storage_dir.is_some() {
        Path::new(".lnk/.roots")
    } else {
        Path::new(".roots")
    };
    let source_path = PathBuf::from(".vagga")
        .join(&roots)
        .join(&ver)
        .join("image.tar.xz");
    match settings.push_image_script {
        Some(ref push_image_script) => {
            let mut upload_cmd = Command::new("/bin/sh");
            upload_cmd.stdin(Stdio::null())
                .arg("-exc")
                .arg(push_image_script)
                .env("source_path", source_path.to_str().unwrap())
                .env("container_name", &opt.name)
                .env("hash", &hash);
            info!("Running {:?}", upload_cmd);
            match upload_cmd.status() {
                Ok(st) if !st.success() => {
                    return Err(format!("Error when uploading image: {:?}", upload_cmd));
                },
                Err(e) => {
                    return Err(format!("Error when uploading image: {}", e));
                },
                _ => {},
            }
        },
        None => {
            return Err(format!("You should specify 'push-image-cmd' setting"));
        },
    }

    try!(remove_file(source_path).map_err(|e| format!("{}", e)));

    Ok(0)
}
