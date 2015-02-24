use std::old_io::{USER_RWX, GROUP_READ, GROUP_EXECUTE, OTHER_READ, OTHER_EXECUTE};
use std::old_io::BufferedReader;
use std::old_io::EndOfFile;
use std::old_io::fs::File;
use std::old_io::fs::{copy, chmod, rename};

use config::builders::UbuntuRepoInfo;
use super::super::context::{BuildContext};
use super::super::context::Distribution::{Ubuntu, Unknown};
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;
use super::super::packages;
use super::generic::run_command;
use container::sha256::{Sha256, Digest};


#[derive(Show)]
pub struct UbuntuInfo {
    pub release: String,
    pub apt_update: bool,
    pub has_universe: bool,
}

pub fn fetch_ubuntu_core(ctx: &mut BuildContext, release: &String)
    -> Result<(), String>
{
    let kind = "core";
    let arch = "amd64";
    let url = format!(concat!(
        "http://cdimage.ubuntu.com/ubuntu-{kind}/{release}/",
        "daily/current/{release}-{kind}-{arch}.tar.gz",
        ), kind=kind, arch=arch, release=release);
    let filename = try!(download_file(ctx, &url[0..]));
    try!(unpack_file(ctx, &filename, &Path::new("/vagga/root"), &[],
        &[Path::new("dev")]));

    try!(init_debian_build(ctx));
    try!(set_mirror(ctx));

    return Ok(());
}

fn set_mirror(ctx: &mut BuildContext) -> Result<(), String> {
    let sources_list = Path::new("/vagga/root/etc/apt/sources.list");
    let mut source = BufferedReader::new(try!(File::open(&sources_list)
        .map_err(|e| format!("Error reading sources.list file: {}", e))));
    let tmp = sources_list.with_extension("tmp");
    try!(File::create(&tmp)
        .and_then(|mut f| {
            loop {
                let line = match source.read_line() {
                    Ok(line) => line,
                    Err(ref e) if e.kind == EndOfFile => {
                        break;
                    }
                    Err(e) => return Err(e),
                };
                try!(f.write(line.replace("http://archive.ubuntu.com/ubuntu/",
                     ctx.settings.ubuntu_mirror.as_slice()).as_bytes()));
            }
            Ok(())
        })
        .map_err(|e| format!("Error writing sources.list file: {}", e)));
    try!(rename(&tmp, &sources_list)
        .map_err(|e| format!("Error renaming sources.list file: {}", e)));
    Ok(())
}

fn init_debian_build(ctx: &mut BuildContext) -> Result<(), String> {
    // Do not attempt to start init scripts
    let policy_file = Path::new("/vagga/root/usr/sbin/policy-rc.d");
    try!(File::create(&policy_file)
        .and_then(|mut f| f.write(b"#!/bin/sh\nexit 101\n"))
        .map_err(|e| format!("Error writing policy-rc.d file: {}", e)));
    try!(chmod(&policy_file,
        USER_RWX|GROUP_READ|GROUP_EXECUTE|OTHER_READ|OTHER_EXECUTE)
        .map_err(|e| format!("Can't chmod file: {}", e)));

    // Do not need to fsync() after package installation
    try!(File::create(
            &Path::new("/vagga/root/etc/dpkg/dpkg.cfg.d/02apt-speedup"))
        .and_then(|mut f| f.write(b"force-unsafe-io"))
        .map_err(|e| format!("Error writing dpkg config: {}", e)));

    // Do not install recommends by default
    try!(File::create(
            &Path::new("/vagga/root/etc/apt/apt.conf.d/01norecommend"))
        .and_then(|mut f| f.write(br#"
            APT::Install-Recommends "0";
            APT::Install-Suggests "0";
        "#))
        .map_err(|e| format!("Error writing apt config: {}", e)));

    // Revert resolv.conf back
    try!(copy(&Path::new("/etc/resolv.conf"),
              &Path::new("/vagga/root/etc/resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    try!(run_command(ctx, &[
        "/usr/sbin/locale-gen".to_string(),
        "en_US.UTF-8".to_string(),
        ]));

    ctx.add_remove_dir(Path::new("/var/lib/apt"));
    // TODO(tailhook) decide if we want to delete package databases
    // ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    return Ok(());
}

pub fn apt_install(ctx: &mut BuildContext, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let apt_update = if let Ubuntu(ref ubuntu) = ctx.distribution {
        ubuntu.apt_update
    } else {
        return Err(format!("Incompatible distribution: {:?}",
                           ctx.distribution));
    };
    if apt_update {
        if let Ubuntu(ref mut ubuntu) = ctx.distribution {
            ubuntu.apt_update = false;
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
        return Err(format!("Incompatible distribution: {:?}",
                           ctx.distribution));
    };

    File::create(&Path::new("/vagga/root/etc/apt/sources.list.d")
                 .join(name.as_slice()))
        .and_then(|mut f| {
            try!(write!(&mut f, "deb {} {}", repo.url, repo.suite));
            for item in repo.components.iter() {
                try!(write!(&mut f, " {}", item));
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
                try!(writeln!(&mut f, "deb {} {} universe",
                    ctx.settings.ubuntu_mirror, ubuntu.release));
                try!(writeln!(&mut f, "deb {} {}-updates universe",
                    ctx.settings.ubuntu_mirror, ubuntu.release));
                try!(writeln!(&mut f, "deb {} {}-security universe",
                    ctx.settings.ubuntu_mirror, ubuntu.release));
                Ok(())
            })
            .map_err(|e| format!("Error writing universe.list file: {}", e)));
    } else {
        return Err(format!("Incompatible distribution: {:?}",
                           ctx.distribution));
    };
    Ok(())
}


fn build_deps(pkg: packages::Package) -> Option<Vec<&'static str>> {
    match pkg {
        packages::BuildEssential => Some(vec!("build-essential")),
        packages::Python2 => Some(vec!()),
        packages::Python2Dev => Some(vec!("python-dev")),
        packages::Python3 => Some(vec!()),
        packages::Python3Dev => Some(vec!("python3-dev")),
        packages::PipPy2 => None,
        packages::PipPy3 => None,
        packages::NodeJs => Some(vec!()),
        packages::NodeJsDev => Some(vec!("nodejs-dev")),
        packages::Npm => Some(vec!("npm")),
        packages::Git => Some(vec!("git")),
        packages::Mercurial => Some(vec!("hg")),
    }
}

fn system_deps(pkg: packages::Package) -> Option<Vec<&'static str>> {
    match pkg {
        packages::BuildEssential => Some(vec!()),
        packages::Python2 => Some(vec!("python")),
        packages::Python2Dev => Some(vec!()),
        packages::Python3 => Some(vec!("python3")),
        packages::Python3Dev => Some(vec!()),
        packages::PipPy2 => None,
        packages::PipPy3 => None,
        packages::NodeJs => Some(vec!("nodejs", "nodejs-legacy")),
        packages::NodeJsDev => Some(vec!()),
        packages::Npm => Some(vec!()),
        packages::Git => Some(vec!()),
        packages::Mercurial => Some(vec!()),
    }
}


pub fn ensure_packages(ctx: &mut BuildContext, features: &[packages::Package])
    -> Result<Vec<packages::Package>, String>
{
    let needs_universe = match ctx.distribution {
        Ubuntu(ref ubuntu) => !ubuntu.has_universe,
        _ => unreachable!(),
    };
    if needs_universe {
        debug!("Add Universe");
        try!(ubuntu_add_universe(ctx));
    }
    let mut to_install = vec!();
    let mut unsupp = vec!();
    for i in features.iter() {
        if let Some(lst) = build_deps(*i) {
            for i in lst.into_iter() {
                if !ctx.packages.contains(i) {
                    ctx.build_deps.insert(i.to_string());
                    to_install.push(i.to_string());
                }
            }
        } else {
            unsupp.push(*i);
            continue;
        }
        if let Some(lst) = system_deps(*i) {
            for i in lst.into_iter() {
                let istr = i.to_string();
                if !ctx.packages.contains(&istr) {
                    ctx.build_deps.remove(&istr);
                }
                if !ctx.packages.contains(&istr) {
                    ctx.packages.insert(istr.clone());
                    to_install.push(istr);
                }
            }
        } else {
            unsupp.push(*i);
            continue;
        }
    }
    if to_install.len() > 0 {
        try!(apt_install(ctx, &to_install));
    }
    return Ok(unsupp);
}
