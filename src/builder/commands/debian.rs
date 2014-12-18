use std::io::fs::File;


use super::super::context::BuildContext;
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;
use super::generic::run_command;


pub fn fetch_ubuntu_core(ctx: &mut BuildContext, release: &String)
    -> Result<(), String>
{
    let kind = "core";
    let arch = "amd64";
    let url = format!(concat!(
        "http://cdimage.ubuntu.com/ubuntu-{kind}/{release}/",
        "daily/current/{release}-{kind}-{arch}.tar.gz",
        ), kind=kind, arch=arch, release=release);
    let filename = try!(download_file(ctx, &url));
    try!(unpack_file(ctx, &filename, &Path::new("/vagga/root"), &[],
        &[Path::new("dev")]));
    try!(init_debian_build(ctx));
    return Ok(());
}

fn init_debian_build(ctx: &mut BuildContext) -> Result<(), String> {

    // Do not attempt to start init scripts
    try!(File::create(
            &Path::new("/vagga/root/usr/sbin/policy-rc.d"))
        .and_then(|mut f| f.write(b"#!/bin/sh\nexit 101\n"))
        .map_err(|e| format!("Error writing policy-rc.d file: {}", e)));

    // Do not need to fsync() after package installation
    try!(File::create(
            &Path::new("/vagga/root/etc/dpkg/dpkg.cfg.d/02apt-speedup"))
        .and_then(|mut f| f.write(b"force-unsafe-io"))
        .map_err(|e| format!("Error writing dpkg config: {}", e)));

    try!(ctx.add_cache_dir(Path::new("/var/cache/apt"),
                           "apt-cache".to_string()));
    // TODO(tailhook) remove apt and dpkg
    ctx.add_remove_dir(Path::new("/var/lib/apt"));
    ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    return Ok(());
}

pub fn apt_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let mut args = vec!(
        "/usr/bin/apt-get".to_string(),
        "install".to_string(),
        "-y".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}
