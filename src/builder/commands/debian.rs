use std::io::fs::File;

use config::builders::UbuntuRepoInfo;
use super::super::context::BuildContext;
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;
use super::generic::run_command;
use container::sha256::{Sha256, Digest};


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

    ctx.ubuntu_release = Some(release.to_string());
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
    ctx.environ.insert("DEBIAN_FRONTEND".to_string(),
                       "noninteractive".to_string());

    ctx.add_remove_dir(Path::new("/var/lib/apt"));
    ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    ctx.apt_update = true;
    return Ok(());
}

pub fn apt_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    if ctx.apt_update {
        ctx.apt_update = false;
        try!(run_command(ctx, &[
            "/usr/bin/apt-get".to_string(),
            "update".to_string(),
            ]));
    }
    let mut args = vec!(
        "/usr/bin/apt-get".to_string(),
        "install".to_string(),
        "-y".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}

pub fn add_debian_repo(ctx: &mut BuildContext, repo: &UbuntuRepoInfo)
    -> Result<(), String>
{

    let mut hash = Sha256::new();
    hash.input_str(repo.url.as_slice());
    hash.input(&[0]);
    hash.input_str(repo.suite.as_slice());
    hash.input(&[0]);
    for cmp in repo.components.iter() {
        hash.input_str(cmp.as_slice());
        hash.input(&[0]);
    }
    let name = format!("{}-{}.list",
        hash.result_str()[..8].to_string(),
        repo.suite);

    ctx.apt_update = true;

    File::create(&Path::new("/vagga/root/etc/apt/sources.list.d")
                 .join(name.as_slice()))
        .and_then(|mut f| {
            try!(write!(f, "deb {} {}", repo.url, repo.suite))
            for item in repo.components.iter() {
                try!(write!(f, " {}", item));
            }
            Ok(())
        })
        .map_err(|e| format!("Error writing {} file: {}", name, e))
    // TODO(tailhook) add `update` command
}

pub fn ubuntu_add_universe(ctx: &mut BuildContext)
    -> Result<(), String>
{
    let release = try!(ctx.ubuntu_release.as_ref()
        .ok_or(format!("Unknown ubuntu release")));
    let target = "/vagga/root/etc/apt/sources.list.d/universe.list";

    ctx.apt_update = true;

    File::create(&Path::new(target))
        .and_then(|mut f| {
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {} universe",
                release));
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {}-updates universe",
                release));
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {}-security universe",
                release));
            Ok(())
        })
        .map_err(|e| format!("Error writing universe.list file: {}", e))
}
