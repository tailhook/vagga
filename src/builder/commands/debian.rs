use std::io::fs::File;
use std::collections::EnumSet;

use config::builders::UbuntuRepoInfo;
use super::super::context::{BuildContext, Ubuntu, Unknown};
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;
use super::generic::run_command;
use super::pip;
use container::sha256::{Sha256, Digest};


#[deriving(Show)]
pub struct UbuntuInfo {
    pub release: String,
    pub apt_update: bool,
    pub has_universe: bool,
}


pub fn fetch_ubuntu_core(ctx: &mut BuildContext, release: &String)
    -> Result<(), String>
{
    if let Unknown = ctx.distribution {
    } else {
        return Err(format!("Conflicting distribution"));
    };
    let kind = "core";
    let arch = "amd64";
    let url = format!(concat!(
        "http://cdimage.ubuntu.com/ubuntu-{kind}/{release}/",
        "daily/current/{release}-{kind}-{arch}.tar.gz",
        ), kind=kind, arch=arch, release=release);
    let filename = try!(download_file(ctx, &url));
    try!(unpack_file(ctx, &filename, &Path::new("/vagga/root"), &[],
        &[Path::new("dev")]));

    ctx.distribution = Ubuntu(UbuntuInfo {
        release: release.to_string(),
        apt_update: true,
        has_universe: false,
    });
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
    try!(ctx.add_cache_dir(Path::new("/var/lib/apt/lists"),
                          "apt-lists".to_string()));
    ctx.environ.insert("DEBIAN_FRONTEND".to_string(),
                       "noninteractive".to_string());
    ctx.environ.insert("LANG".to_string(),
                       "en_US.UTF-8".to_string());
    try!(run_command(ctx, &[
        "/usr/sbin/locale-gen".to_string(),
        "en_US.UTF-8".to_string(),
        ]));

    ctx.add_remove_dir(Path::new("/var/lib/apt"));
    ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    return Ok(());
}

pub fn apt_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let apt_update = if let Ubuntu(ref ubuntu) = ctx.distribution {
        ubuntu.apt_update
    } else {
        return Err(format!("Incompatible distribution: {}", ctx.distribution));
    };
    if apt_update {
        if let Ubuntu(ref mut ubuntu) = ctx.distribution {
            ubuntu.apt_update = true;
        }
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

pub fn apt_remove(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let mut args = vec!(
        "/usr/bin/apt-get".to_string(),
        "remove".to_string(),
        "-y".to_string(),
        );
    args.extend(pkgs.clone().into_iter());
    run_command(ctx, args.as_slice())
}

pub fn finish(ctx: &mut BuildContext) -> Result<(), String>
{
    let pkgs = ctx.build_deps.clone().into_iter().collect();
    try!(apt_remove(ctx, &pkgs));
    run_command(ctx, &[
        "/usr/bin/apt-get".to_string(),
        "autoremove".to_string(),
        "-y".to_string(),
        ])
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

    if let Ubuntu(ref mut ubuntu) = ctx.distribution {
        ubuntu.apt_update = true;
    } else {
        return Err(format!("Incompatible distribution: {}", ctx.distribution));
    };

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
    let target = "/vagga/root/etc/apt/sources.list.d/universe.list";
    if let Ubuntu(ref ubuntu) = ctx.distribution {
        try!(File::create(&Path::new(target))
            .and_then(|mut f| {
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {} universe",
                ubuntu.release));
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {}-updates universe",
                ubuntu.release));
            try!(writeln!(f,
                "deb http://archive.ubuntu.com/ubuntu/ {}-security universe",
                ubuntu.release));
                Ok(())
            })
            .map_err(|e| format!("Error writing universe.list file: {}", e)));
    } else {
        return Err(format!("Incompatible distribution: {}", ctx.distribution));
    };
    match ctx.distribution {
        Ubuntu(ref mut ubuntu) => { ubuntu.has_universe = true; }
        _ => unreachable!(),
    }
    Ok(())
}

pub fn ensure_pip(ctx: &mut BuildContext, ver: u8,
    features: &EnumSet<pip::PipFeatures>)
    -> Result<Path, String>
{
    let needs_universe = match ctx.distribution {
        Ubuntu(ref ubuntu) => !ubuntu.has_universe,
        _ => unreachable!(),
    };
    if needs_universe {
        debug!("Add Universe");
        try!(ubuntu_add_universe(ctx));
    }
    let mut packages = vec!(
        (if ver == 2 { "python" } else { "python3" }).to_string(),
        );
    ctx.packages.extend(packages.clone().into_iter());
    for i in features.iter() {
        let dep = match i {
            pip::Dev => (if ver == 2 { "python-dev" }
                         else { "python3-dev" }).to_string(),
            pip::Pip => (if ver == 2 { "python-pip" }
                         else { "python3-pip" }).to_string(),
            pip::Git => "git".to_string(),
            pip::Hg => "hg".to_string(),
        };
        if !ctx.packages.contains(&dep) {
            if ctx.build_deps.insert(dep.clone()) {
                packages.push(dep);
            }
        }
    }
    if packages.len() > 0 {
        try!(apt_install(ctx, &packages));
    }
    return Ok(Path::new(format!("/usr/bin/pip{}", ver)));
}
