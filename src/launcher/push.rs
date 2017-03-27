use std::fs::remove_file;
use std::path::{Path, PathBuf};

use unshare::{Command, Stdio};

use process_util::{run_success};
use launcher::Context;
use launcher::build::build_container;
use launcher::wrap::Wrapper;
use launcher::storage_dir;
use options::push::Options;


pub fn push_command(ctx: &Context, args: Vec<String>) -> Result<i32, String>
{
    let mut cmdline = args.clone();
    cmdline.insert(0, "vagga _push_image".to_string());
    let opt = match Options::parse(&cmdline) {
        Ok(x) => x,
        Err(code) => return Ok(code),
    };

    let cinfo = ctx.config.get_container(&opt.name)?;

    let ver = build_container(ctx, &opt.name, opt.build_mode, false)?;
    let short_hash = match ver.rsplitn(2, ".").next() {
        Some(v) => v,
        None => return Err(format!("Incorrect container version")),
    };

    let mut pack_cmd: Command = Wrapper::new(Some(&ver), &ctx.settings);
    let image_name = "image.tar.xz";
    let image_path = Path::new("/vagga/base/.roots")
        .join(&ver)
        .join(image_name);
    pack_cmd.map_users_for(cinfo, &ctx.settings)?;
    pack_cmd.gid(0);
    pack_cmd.groups(Vec::new());
    pack_cmd
        .arg("_pack_image")
        .arg(&opt.name)
        .arg("-f").arg(&image_path)
        .arg("-J");
    run_success(pack_cmd)?;

    let roots = storage_dir::get_base(ctx).map(|x| x.join(".roots"))
        .expect("storage dir created by preceding container build");
    let tmp_image_path = PathBuf::from(".vagga")
        .join(&roots)
        .join(&ver)
        .join("image.tar.xz");
    match ctx.settings.push_image_script {
        Some(ref push_image_script) => {
            let mut upload_cmd = Command::new("/bin/sh");
            upload_cmd.stdin(Stdio::null())
                .arg("-exc")
                .arg(push_image_script)
                .env("image_path", tmp_image_path.to_str().unwrap())
                .env("container_name", &opt.name)
                .env("short_hash", &short_hash);
            run_success(upload_cmd)?;
        },
        None => {
            return Err(format!("You should specify 'push-image-script' setting"));
        },
    }

    remove_file(tmp_image_path).map_err(|e| format!("{}", e))?;

    Ok(0)
}
