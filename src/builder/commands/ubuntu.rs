use std::fs::{rename, set_permissions, Permissions};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::os::unix::io::{AsRawFd, FromRawFd};

use config::builders::{UbuntuRepoInfo, UbuntuReleaseInfo, AptKey};
use container::util::clean_dir;
use super::super::context::{Context};
use super::super::download::download_file;
use super::super::tarcmd::unpack_file;
use super::super::packages;
use builder::commands::generic::{command, run};
use shaman::sha2::Sha256;
use shaman::digest::Digest;
use builder::distrib::{Distribution, Named, DistroBox};
use builder::guard::Guard;
use builder::error::StepError;
use unshare::Stdio;
use file_util::copy;

#[derive(Debug)]
pub enum Version {
    Daily { codename: String },
    Release { version: String },
}

#[derive(Debug)]
pub struct Ubuntu {
    version: Version,
    codename: Option<String>,
    arch: String,
    apt_update: bool,
    has_universe: bool,
    clobber_chfn: bool,
}

impl Named for Ubuntu {
    fn static_name() -> &'static str { "ubuntu" }
}

impl Distribution for Ubuntu {
    fn name(&self) -> &'static str { "Ubuntu" }
    fn bootstrap(&mut self, ctx: &mut Context) -> Result<(), StepError> {
        try!(fetch_ubuntu_core(ctx, &self.version, self.arch.clone()));
        let codename = try!(read_ubuntu_codename());
        if self.codename.is_some() && self.codename.as_ref() != Some(&codename) {
            return Err(From::from("Codename mismatch. \
                This is either bug of vagga or may be damaged archive"));
        }
        ctx.binary_ident = format!("{}-ubuntu-{}",
            ctx.binary_ident, codename);
        try!(init_ubuntu_core(ctx));
        if self.clobber_chfn {
            try!(clobber_chfn());
        }
        Ok(())
    }
    fn install(&mut self, ctx: &mut Context, pkgs: &[String])
        -> Result<(), StepError>
    {
        if self.apt_update {
            self.apt_update = false;
            let mut cmd = try!(command(ctx, "apt-get"));
            cmd.arg("update");
            try!(run(cmd)
                .map_err(|error| {
                    debug!("The apt-get update failed. \
                        Cleaning apt-lists so that apt can proceed next time");
                    clean_dir(&Path::new("/vagga/cache/apt-lists"), false)
                        .map_err(|e| error!(
                            "Cleaning apt-lists cache failed: {}", e)).ok();
                    error
                }));
        }
        let mut cmd = try!(command(ctx, "apt-get"));
        cmd.arg("install");
        cmd.arg("-y");
        cmd.args(&pkgs[..]);
        try!(run(cmd));
        Ok(())
    }
    fn ensure_packages(&mut self, ctx: &mut Context,
        features: &[packages::Package])
        -> Result<Vec<packages::Package>, StepError>
    {
        if !self.has_universe {
            debug!("Add Universe");
            try!(self.enable_universe());
            try!(self.add_universe(ctx));
        }
        let mut to_install = vec!();
        let mut unsupp = vec!();
        for i in features.iter() {
            if let Some(lst) = build_deps(*i) {
                for i in lst.into_iter() {
                    if !ctx.packages.contains(i) {
                        if ctx.build_deps.insert(i.to_string()) {
                            to_install.push(i.to_string());
                        }
                    }
                }
            } else {
                unsupp.push(*i);
                continue;
            }
            if let Some(lst) = self.system_deps(*i) {
                for i in lst.into_iter() {
                    let istr = i.to_string();
                    ctx.build_deps.remove(&istr);
                    if ctx.packages.insert(istr.clone()) {
                        to_install.push(istr);
                    }
                }
            } else {
                unsupp.push(*i);
                continue;
            }
        }
        if to_install.len() > 0 {
            try!(self.install(ctx, &to_install));
        }
        return Ok(unsupp);
    }
    fn finish(&mut self, ctx: &mut Context) -> Result<(), String>
    {
        let pkgs: Vec<_> = ctx.build_deps.clone().into_iter().collect();
        if pkgs.len() > 0 {
            let mut cmd = try!(command(ctx, "apt-mark"));
            cmd.arg("auto");
            cmd.args(&pkgs[..]);
            try!(run(cmd));
        }
        let mut cmd = try!(command(ctx, "apt-get"));
        cmd.arg("autoremove").arg("-y");
        try!(run(cmd));

        let pkglist = "/vagga/container/debian-packages.txt";
        let output = try!(File::create(pkglist)
            .map_err(|e| StepError::Write(PathBuf::from(pkglist), e)));
        let mut cmd = try!(command(ctx, "dpkg"));
        cmd.arg("-l");
        // TODO(tailhook) fixme in rust 1.6. as_raw_fd -> into_raw_fd
        cmd.stdout(unsafe { Stdio::from_raw_fd(output.as_raw_fd()) });
        try!(run(cmd));
        Ok(())
    }
}

impl Ubuntu {
    pub fn enable_universe(&mut self) -> Result<(), StepError> {
        self.has_universe = true;
        self.apt_update = true;
        Ok(())
    }
    pub fn add_debian_repo(&mut self, _: &mut Context, repo: &UbuntuRepoInfo)
        -> Result<(), String>
    {
        self.apt_update = true;

        let mut hash = Sha256::new();
        hash.input_str(&repo.url);
        hash.input(&[0]);
        hash.input_str(&repo.suite);
        hash.input(&[0]);
        for cmp in repo.components.iter() {
            hash.input_str(&cmp);
            hash.input(&[0]);
        }
        let name = format!("{}-{}.list",
            hash.result_str()[..8].to_string(),
            repo.suite);

        File::create(&Path::new("/vagga/root/etc/apt/sources.list.d")
                     .join(&name))
            .and_then(|mut f| {
                try!(write!(&mut f, "deb {} {}", repo.url, repo.suite));
                for item in repo.components.iter() {
                    try!(write!(&mut f, " {}", item));
                }
                Ok(())
            })
            .map_err(|e| format!("Error writing {} file: {}", name, e))
    }
    pub fn add_ubuntu_ppa(&mut self, ctx: &mut Context, name: &str)
        -> Result<(), StepError>
    {
        try!(self.ensure_codename(ctx));
        let suite = self.codename.as_ref().unwrap().to_string();
        try!(self.add_debian_repo(ctx, &UbuntuRepoInfo {
            url: format!("http://ppa.launchpad.net/{}/ubuntu", name),
            suite: suite,
            components: vec!["main".to_string()],
        }));
        Ok(())
    }
    pub fn add_apt_key(&mut self, ctx: &mut Context, key: &AptKey)
        -> Result<(), StepError>
    {
        let mut cmd = try!(command(ctx, "apt-key"));
        cmd.arg("adv");
        cmd.arg("--keyserver");
        if let Some(ref srv) = key.server {
            cmd.arg(srv);
        } else {
            cmd.arg("keyserver.ubuntu.com");
        }
        cmd.arg("--recv-keys");
        for item in &key.keys {
            cmd.arg(item);
        }
        run(cmd)
    }
    pub fn ensure_codename(&mut self, ctx: &mut Context)
        -> Result<(), StepError>
    {
        if self.codename.is_none() {
            let codename = try!(read_ubuntu_codename());
            ctx.binary_ident = format!("{}-ubuntu-{}",
                ctx.binary_ident, codename);
            self.codename = Some(codename);
        }
        Ok(())
    }

    pub fn add_universe(&mut self, ctx: &mut Context)
        -> Result<(), String>
    {
        try!(self.ensure_codename(ctx));
        let codename = self.codename.as_ref().unwrap();
        let target = "/vagga/root/etc/apt/sources.list.d/universe.list";
        try!(File::create(&Path::new(target))
            .and_then(|mut f| {
                try!(writeln!(&mut f, "deb {} {} universe",
                    ctx.settings.ubuntu_mirror, codename));
                try!(writeln!(&mut f, "deb {} {}-updates universe",
                    ctx.settings.ubuntu_mirror, codename));
                try!(writeln!(&mut f, "deb {} {}-security universe",
                    ctx.settings.ubuntu_mirror, codename));
                Ok(())
            })
            .map_err(|e| format!("Error writing universe.list file: {}", e)));
        Ok(())
    }
    fn needs_node_legacy(&self) -> bool {
        self.codename.as_ref().map(|x| &x[..] != "precise").unwrap_or(false)
    }
    fn system_deps(&self, pkg: packages::Package) -> Option<Vec<&'static str>> {
        match pkg {
            packages::BuildEssential => Some(vec!()),
            packages::Https => Some(vec!()),
            packages::Python2 => Some(vec!("python")),
            packages::Python2Dev => Some(vec!()),
            packages::Python3 => Some(vec!("python3")),
            packages::Python3Dev => Some(vec!()),
            packages::PipPy2 => None,
            packages::PipPy3 => None,
            packages::NodeJs if self.needs_node_legacy() => {
                Some(vec!("nodejs", "nodejs-legacy"))
            }
            packages::NodeJs => Some(vec!("nodejs")),
            packages::NodeJsDev => Some(vec!()),
            packages::Npm => Some(vec!()),
            packages::Git => Some(vec!()),
            packages::Mercurial => Some(vec!()),
        }
    }
}

pub fn read_ubuntu_codename() -> Result<String, String>
{
    let lsb_release_path = "/vagga/root/etc/lsb-release";
    let lsb_release_file = BufReader::new(
        try_msg!(File::open(&Path::new(lsb_release_path)),
            "Error reading /etc/lsb-release: {err}"));

    for line in lsb_release_file.lines() {
        let line = try_msg!(line, "Error reading lsb file: {err}");
        if let Some(equals_pos) = line.find('=') {
            let key = line[..equals_pos].trim();

            if key == "DISTRIB_CODENAME" {
                let value = line[(equals_pos + 1)..].trim();
                return Ok(value.to_string());
            }
        }
    }

    Err(format!("Coudn't read codename from '{lsb_release_path}'",
                lsb_release_path=lsb_release_path))
}

pub fn fetch_ubuntu_core(ctx: &mut Context, ver: &Version, arch: String)
    -> Result<(), String>
{
    let url_base = "http://cdimage.ubuntu.com/ubuntu";
    let kind = "core";
    let url = match *ver {
        Version::Daily { ref codename } => {
            format!(
                "{url_base}-{kind}/{release}/daily/current/\
                 {release}-{kind}-{arch}.tar.gz",
                url_base=url_base, kind=kind, arch=arch, release=codename)
        },
        Version::Release { ref version } => {
            format!(
                "{url_base}-{kind}/releases/{release}/release/\
                 ubuntu-{kind}-{release}-{kind}-{arch}.tar.gz",
                url_base=url_base, kind=kind, arch=arch, release=version)
        },
    };
    let filename = try!(download_file(ctx, &url[0..]));
    try!(unpack_file(ctx, &filename, &Path::new("/vagga/root"), &[],
        &[Path::new("dev")]));

    Ok(())
}

pub fn init_ubuntu_core(ctx: &mut Context) -> Result<(), String> {
    try!(init_debian_build(ctx));
    try!(set_mirror(ctx));

    Ok(())
}

fn set_mirror(ctx: &mut Context) -> Result<(), String> {
    let sources_list = Path::new("/vagga/root/etc/apt/sources.list");
    let source = BufReader::new(try!(File::open(&sources_list)
        .map_err(|e| format!("Error reading sources.list file: {}", e))));
    let tmp = sources_list.with_extension("tmp");
    try!(File::create(&tmp)
        .and_then(|mut f| {
            for line in source.lines() {
                let line = try!(line);
                try!(f.write_all(
                    line.replace("http://archive.ubuntu.com/ubuntu/",
                     &ctx.settings.ubuntu_mirror).as_bytes()));
                try!(f.write_all(b"\n"));
            }
            Ok(())
        })
        .map_err(|e| format!("Error writing sources.list file: {}", e)));
    try!(rename(&tmp, &sources_list)
        .map_err(|e| format!("Error renaming sources.list file: {}", e)));
    Ok(())
}

fn init_debian_build(ctx: &mut Context) -> Result<(), String> {
    // Do not attempt to start init scripts
    let policy_file = Path::new("/vagga/root/usr/sbin/policy-rc.d");
    try!(File::create(&policy_file)
        .and_then(|mut f| f.write_all(b"#!/bin/sh\nexit 101\n"))
        .map_err(|e| format!("Error writing policy-rc.d file: {}", e)));
    try!(set_permissions(&policy_file, Permissions::from_mode(0o755))
        .map_err(|e| format!("Can't chmod file: {}", e)));

    // Do not need to fsync() after package installation
    try!(File::create(
            &Path::new("/vagga/root/etc/dpkg/dpkg.cfg.d/02apt-speedup"))
        .and_then(|mut f| f.write_all(b"force-unsafe-io"))
        .map_err(|e| format!("Error writing dpkg config: {}", e)));

    // Do not install recommends by default
    try!(File::create(
            &Path::new("/vagga/root/etc/apt/apt.conf.d/01norecommend"))
        .and_then(|mut f| f.write_all(br#"
            APT::Install-Recommends "0";
            APT::Install-Suggests "0";
        "#))
        .map_err(|e| format!("Error writing apt config: {}", e)));

    // Revert resolv.conf back
    try!(copy(&Path::new("/etc/resolv.conf"),
              &Path::new("/vagga/root/etc/resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    let mut cmd = try!(command(ctx, "locale-gen"));
    cmd.arg("en_US.UTF-8");
    try!(run(cmd));

    // TODO(tailhook) reconsider this. It was fun to remove unneeded files
    //                until we have !Container which fails ot reuse ubuntu
    //                container when /var/lib/apt is clean
    // ctx.add_remove_dir(Path::new("/var/lib/apt"));
    // TODO(tailhook) decide if we want to delete package databases
    // ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
    return Ok(());
}

fn build_deps(pkg: packages::Package) -> Option<Vec<&'static str>> {
    match pkg {
        packages::BuildEssential => Some(vec!("build-essential")),
        packages::Https => Some(vec!("ca-certificates")),
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


pub fn clobber_chfn() -> Result<(), String> {
    try_msg!(symlink("/bin/true", "/vagga/root/usr/bin/.tmp.chfn"),
        "Can't clobber chfn (symlink error): {err}");
    try_msg!(rename("/vagga/root/usr/bin/.tmp.chfn",
                    "/vagga/root/usr/bin/chfn"),
        "Can't clobber chfn (rename error): {err}");
    Ok(())
}

pub fn configure(guard: &mut Guard, info: &UbuntuReleaseInfo)
    -> Result<(), StepError>
{
    try!(guard.distro.set(Ubuntu {
        version: Version::Release { version: info.version.clone() },
        arch: info.arch.clone(),
        codename: None, // unknown yet
        apt_update: true,
        has_universe: false,
        clobber_chfn: !info.keep_chfn_command,
    }));
    configure_common(&mut guard.ctx)
}

fn configure_common(ctx: &mut Context) -> Result<(), StepError> {
    try!(ctx.add_cache_dir(Path::new("/var/cache/apt"),
                           "apt-cache".to_string()));
    try!(ctx.add_cache_dir(Path::new("/var/lib/apt/lists"),
                          "apt-lists".to_string()));
    ctx.environ.insert("DEBIAN_FRONTEND".to_string(),
                       "noninteractive".to_string());
    ctx.environ.insert("LANG".to_string(),
                       "en_US.UTF-8".to_string());
    ctx.environ.insert("PATH".to_string(),
                       "/usr/local/sbin:/usr/local/bin:\
                        /usr/sbin:/usr/bin:/sbin:/bin:\
                        /usr/games:/usr/local/games\
                        ".to_string());
    Ok(())
}

pub fn configure_simple(guard: &mut Guard, codename: &str)
    -> Result<(), StepError>
{
    try!(guard.distro.set(Ubuntu {
        version: Version::Daily { codename: codename.to_string() },
        arch: "amd64".to_string(),
        codename: Some(codename.to_string()),
        clobber_chfn: true,
        apt_update: true,
        has_universe: false,
    }));
    configure_common(&mut guard.ctx)
}
