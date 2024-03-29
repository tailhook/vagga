use std::fs::{remove_dir_all, rename, set_permissions, Permissions};
use std::os::unix::fs::{symlink};
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

#[cfg(feature="containers")] use scan_dir::ScanDir;
#[cfg(feature="containers")] use unshare::Stdio;
use quire::validate as V;
use lazy_static::lazy_static;

#[cfg(feature="containers")]
use crate::{
    builder::commands::generic::{command, run, find_cmd},
    builder::commands::tarcmd::unpack_file,
    builder::context::Context,
    builder::distrib::{Distribution, DistroBox, Named},
    builder::dns::revert_name_files,
    builder::packages,
    container::util::clean_dir,
    file_util::{CopyTimePolicy, Dir, FileCopy, Lock, safe_remove},
};
use crate::build_step::{BuildStep, Config, Digest, Guard, StepError, VersionError};

#[cfg(feature="containers")]
use self::build::*;

lazy_static! {
    static ref APT_CACHE: &'static Path = Path::new("/var/cache/apt");
}
const APT_CACHE_NAME: &str = "apt-cache";

// Build Steps
#[derive(Debug, Serialize, Deserialize)]
pub struct Ubuntu(String);

#[derive(Debug)]
struct EMDParams {
    needs_universe: bool,
    package: &'static str,
}

impl Ubuntu {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UbuntuRelease {
    pub codename: Option<String>,
    pub version: Option<String>,
    pub url: Option<String>,
    pub arch: String,
    pub keep_chfn_command: bool,
    pub eatmydata: bool,
}

impl UbuntuRelease {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("version", V::Scalar::new().optional())
        .member("codename", V::Scalar::new().optional())
        .member("url", V::Scalar::new().optional())
        .member("arch", V::Scalar::new().default("amd64"))
        .member("keep_chfn_command", V::Scalar::new().default(false))
        .member("eatmydata", V::Scalar::new().default(true))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UbuntuUniverse;

impl UbuntuUniverse {
    pub fn config() -> V::Nothing {
        V::Nothing
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UbuntuPPA(String);

impl UbuntuPPA {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UbuntuRepo {
    pub url: Option<String>,
    pub suite: Option<String>,
    pub components: Vec<String>,
    pub trusted: bool,
}

impl UbuntuRepo {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new().optional())
        .member("suite", V::Scalar::new().optional())
        .member("components", V::Sequence::new(V::Scalar::new()))
        .member("trusted", V::Scalar::new().default(false))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AptTrust {
    pub server: Option<String>,
    pub keys: Vec<String>,
}

impl AptTrust {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("server", V::Scalar::new().optional())
        .member("keys", V::Sequence::new(V::Scalar::new()))
    }
}

#[derive(Debug, PartialEq)]
enum AptHkps {
    No,
    Installed,
}

#[derive(Debug, PartialEq)]
enum AptHttps {
    No,
    Need,
    Installed,
}

#[derive(Debug, PartialEq)]
enum EatMyData {
    No,
    Need,
    Preload(String),
}

#[derive(Debug)]
pub struct Distro {
    config: UbuntuRelease,
    pub codename: Option<String>,
    apt_update: bool,
    apt_https: AptHttps,
    apt_hkps: AptHkps,
    has_indices: bool,
    has_universe: bool,
    eatmydata: EatMyData,
}

#[cfg(feature="containers")]
impl Named for Distro {
    fn static_name() -> &'static str { "ubuntu" }
}

#[cfg(feature="containers")]
impl Distribution for Distro {
    fn name(&self) -> &'static str { "Ubuntu" }
    #[cfg(feature="containers")]
    fn bootstrap(&mut self, ctx: &mut Context) -> Result<(), StepError> {
        fetch_ubuntu_core(ctx, &self.config)?;
        let codename = read_ubuntu_codename()?;
        if self.codename.is_some() && self.codename.as_ref() != Some(&codename) {
            return Err(From::from("Codename mismatch. \
                This is either bug of vagga or may be damaged archive"));
        }
        if self.codename.is_none() {
            self.codename = Some(codename.clone());
        }
        ctx.binary_ident = format!("{}-ubuntu-{}",
            ctx.binary_ident, codename);
        init_ubuntu_core(ctx, self)?;
        if !self.config.keep_chfn_command {
            clobber_chfn()?;
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn add_repo(&mut self, ctx: &mut Context, repo: &str)
        -> Result<(), StepError>
    {
        let repo_parts = repo.split('/').collect::<Vec<_>>();
        let (suite, component) = match repo_parts.len() {
            1 => {
                (self.get_codename(ctx)?.to_string(), repo_parts[0].to_string())
            },
            2 => (repo_parts[0].to_string(), repo_parts[1].to_string()),
            _ => {
                return Err(StepError::from(format!(
                    "Cannot parse repository string. \
                     Should be in the next formats: \
                     'suite/component' or 'component'. \
                     But was: '{}'", repo)));
            },
        };
        let ubuntu_repo = UbuntuRepo {
            url: None,
            suite: Some(suite),
            components: vec!(component),
            trusted: false,
        };
        self.add_debian_repo(ctx, &ubuntu_repo)?;
        Ok(())
    }
    #[cfg(feature="containers")]
    fn install(&mut self, ctx: &mut Context, pkgs: &[String])
        -> Result<(), StepError>
    {
        if self.apt_update {
            if self.apt_https == AptHttps::Need {
                self.install_apt_deps(ctx, &["ca-certificates", "apt-transport-https"])?;
                self.apt_https = AptHttps::Installed;
            }
            if !self.has_indices {
                self.ensure_codename(ctx)?;
                self.copy_apt_lists_from_cache()
                .map_err(|e| error!("Error copying apt-lists cache: {}. \
                    Ignored.", e)).ok();
                self.has_indices = true;
            }
            self.ensure_basic_packages(ctx, &[])?;
            // may be already updated by eatmydata
            apt_get_update::<&str>(ctx, &[], &mut self.apt_update)?;
        }
        apt_get_install(ctx, &pkgs[..], &self.eatmydata)?;
        Ok(())
    }
    #[cfg(feature="containers")]
    fn ensure_packages(&mut self, ctx: &mut Context,
        features: &[packages::Package])
        -> Result<Vec<packages::Package>, StepError>
    {
        if !self.has_universe {
            debug!("Add Universe for ensure packages");
            self.enable_universe()?;
            self.add_universe(ctx)?;
        }
        let mut to_install = vec!();
        let mut unsupp = vec!();
        for i in features.iter() {
            if let Some(lst) = self.build_deps(*i) {
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
            if let Some(lst) = self.system_deps(*i, ctx) {
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
            self.install(ctx, &to_install)?;
        }
        return Ok(unsupp);
    }
    #[cfg(feature="containers")]
    fn finish(&mut self, ctx: &mut Context) -> Result<(), String>
    {
        let pkgs: Vec<String> = ctx.build_deps.iter()
            .map(|p| {
                p.splitn(2, '=')
                    .next()
                    .map(|n| n.to_string())
                    .unwrap_or(p.clone())
            })
            .collect();
        if pkgs.len() > 0 {
            let mut cmd = command(ctx, "apt-mark")?;
            cmd.arg("auto");
            cmd.args(&pkgs[..]);
            run(cmd)?;
        }
        let mut cmd = command(ctx, "apt-get")?;
        build::eat_my_data(&mut cmd, ctx, &self.eatmydata);
        cmd.arg("-oDir::cache::pkgcache=");
        cmd.arg("-oDir::cache::srcpkgcache=");
        if ctx.settings.ubuntu_skip_locking {
            cmd.arg("-oDebug::NoLocking=yes");
        }
        cmd.arg("autoremove").arg("-y");

        {
            let _lock = apt_get_lock(
                !ctx.settings.ubuntu_skip_locking &&
                !ctx.is_cached_dir_overlay(&APT_CACHE)
            );
            run(cmd)?;
        }

        let pkglist = "/vagga/container/debian-packages.txt";
        let output = File::create(pkglist)
            .map_err(|e| StepError::Write(PathBuf::from(pkglist), e))?;
        let mut cmd = command(ctx, "dpkg")?;
        cmd.arg("-l");
        cmd.stdout(Stdio::from_file(output));
        run(cmd)
            .map_err(|e| warn!("Can't list debian packages: {}", e)).ok();

        if let Some(cached_dir) = ctx.get_cached_dir(&APT_CACHE) {
            if let Some(diff_dir) = cached_dir.diff.as_ref() {
                self.copy_apt_archives_to_cache(diff_dir, cached_dir.src.as_path())
                    .map_err(|e|
                        error!("error when copying archives to cache: {}. Ignored", e)
                    )
                    .ok();
            }
        }

        self.copy_apt_lists_to_cache()
            .map_err(|e| error!("error when caching apt-lists: {}. Ignored.",
                e)).ok();
        // Remove lists after copying to cache, for two reasons:
        // 1. It occupies space that is useless after installation
        // 2. `partial` subdir has limited permissions, so you need to deal
        //    with it when rsyncing directory to production
        clean_dir("/vagga/root/var/lib/apt/lists", false)?;

        // This is the only directory in standard distribution that has
        // permissions of 0o700. While it's find for vagga itself it keeps
        // striking us when rsyncing containers to production (i.e. need to
        // remove the directory everywhere). But the directory is just useless
        // in 99.9% cases because nobody wants to run rsyslog in container.
        clean_dir("/vagga/root/var/spool/rsyslog", true)?;

        // Remove log files
        for log_path in &["/vagga/root/var/log/alternatives.log",
                          "/vagga/root/var/log/bootstrap.log",
                          "/vagga/root/var/log/dpkg.log",
                          "/vagga/root/var/log/apt/history.log",
                          "/vagga/root/var/log/apt/term.log"] {
            try_msg!(safe_remove(log_path),
                "Error when removing log file {path}: {err}", path=log_path);
        }

        Ok(())
    }
}

impl Distro {
    pub fn enable_universe(&mut self) -> Result<(), StepError> {
        self.has_universe = true;
        self.apt_update = true;
        Ok(())
    }
    #[cfg(feature="containers")]
    pub fn add_debian_repo(&mut self, ctx: &mut Context, repo: &UbuntuRepo)
        -> Result<(), String>
    {
        self.apt_update = true;

        let suite = match repo.suite {
            Some(ref suite) => suite,
            None => {
                self.get_codename(ctx)?
            },
        };
        let mirror = ctx.settings.ubuntu_mirror();
        let url = repo.url.as_ref().map(|u| u.as_str()).unwrap_or(mirror);

        let mut hash = Digest::new(false, false);
        hash.opt_field("url", &repo.url);
        hash.field("suite", suite);
        hash.field("components", &repo.components);
        let name = format!("{:.8x}-{}.list", hash, suite);

        File::create(&Path::new("/vagga/root/etc/apt/sources.list.d")
                     .join(&name))
            .and_then(|mut f| {
                let flags = if repo.trusted { " [trusted=yes] " } else { " " };
                write!(&mut f, "deb{}{} {}", flags, url, suite)?;
                for item in repo.components.iter() {
                    write!(&mut f, " {}", item)?;
                }
                Ok(())
            })
            .map_err(|e| format!("Error writing {} file: {}", name, e))
    }
    #[cfg(feature="containers")]
    pub fn add_ubuntu_ppa(&mut self, ctx: &mut Context, name: &str)
        -> Result<(), StepError>
    {
        let suite = self.get_codename(ctx)?.to_string();
        self.add_debian_repo(ctx, &UbuntuRepo {
            url: Some(format!("http://ppa.launchpad.net/{}/ubuntu", name)),
            suite: Some(suite),
            components: vec!["main".to_string()],
            trusted: false,
        })?;
        Ok(())
    }
    #[cfg(feature="containers")]
    fn install_apt_deps(&mut self, ctx: &mut Context, apt_deps: &[&str])
        -> Result<(), StepError>
    {
        for dep in apt_deps.iter() {
            ctx.build_deps.insert(dep.to_string());
        }
        apt_get_update(ctx, &[
            "--no-list-cleanup",
            "-o", "Dir::Etc::sourcelist=sources.list",
            "-o", "Dir::Etc::sourceparts=-"
            ], &mut true)?;
        if self.has_universe {
            apt_get_update(ctx, &[
                "--no-list-cleanup",
                "-o", "Dir::Etc::sourcelist=sources.list.d/universe.list",
                "-o", "Dir::Etc::sourceparts=-"
                ], &mut true)?;
        }
        apt_get_install(ctx, &apt_deps[..], &self.eatmydata)
    }
    #[cfg(feature="containers")]
    pub fn add_apt_key(&mut self, ctx: &mut Context, key: &AptTrust)
        -> Result<(), StepError>
    {
        let mut cmd = command(ctx, "apt-key")?;
        cmd.arg("adv");
        cmd.arg("--keyserver");
        let url = {
            if let Some(ref srv) = key.server {
                srv
            } else {
                "hkps://keyserver.ubuntu.com:443"
            }
        };
        if url.starts_with("hkps:") {
            if AptHkps::Installed != self.apt_hkps {
                if !self.has_universe {
                    debug!("Add Universe for ensure packages");
                    self.enable_universe()?;
                    self.add_universe(ctx)?;
                }
                let packages = match self.get_codename(ctx)? {
                    "precise" => &["gnupg-curl", "ca-certificates"],
                    "trusty" => &["gnupg-curl", "ca-certificates"],
                    "xenial" => &["gnupg-curl", "ca-certificates"],
                    _ /* >= bionic */ => &["gnupg", "ca-certificates"],
                };
                self.install_apt_deps(ctx, packages)?;
                self.apt_hkps = AptHkps::Installed;
            }
        }
        cmd.arg(url);
        cmd.arg("--recv-keys");
        for item in &key.keys {
            cmd.arg(item);
        }
        run(cmd)
    }
    #[cfg(feature="containers")]
    pub fn ensure_codename(&mut self, ctx: &mut Context)
        -> Result<(), StepError>
    {
        if self.codename.is_none() {
            let codename = read_ubuntu_codename()?;
            ctx.binary_ident = format!("{}-ubuntu-{}",
                ctx.binary_ident, codename);
            self.codename = Some(codename);
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    pub fn get_codename(&mut self, ctx: &mut Context)
        -> Result<&str, StepError>
    {
        self.ensure_codename(ctx)?;
        Ok(self.codename.as_ref().unwrap())
    }

    #[cfg(feature="containers")]
    pub fn add_universe(&mut self, ctx: &mut Context)
        -> Result<(), String>
    {
        let codename = self.get_codename(ctx)?;
        let target = "/vagga/root/etc/apt/sources.list.d/universe.list";
        let mirror = ctx.settings.ubuntu_mirror();
        File::create(&Path::new(target))
            .and_then(|mut f| {
                writeln!(&mut f, "deb {} {} universe",
                    mirror, codename)?;
                writeln!(&mut f, "deb {} {}-updates universe",
                    mirror, codename)?;
                writeln!(&mut f, "deb {} {}-security universe",
                    mirror, codename)?;
                Ok(())
            })
            .map_err(|e| format!("Error writing universe.list file: {}", e))?;
        Ok(())
    }
    fn needs_node_legacy(&self) -> bool {
        self.codename.as_ref().map(|x| &x[..] != "precise").unwrap_or(false)
    }
    fn has_php7(&self) -> bool {
        let php5_only = ["precise", "trusty", "vivid", "wily"];
        self.codename.as_ref().map(|cn| !php5_only.contains(&cn.as_ref())).unwrap_or(false)
    }
    fn needs_rubygems(&self) -> bool {
        self.codename.as_ref().map(|cn| cn == "precise").unwrap_or(false)
    }

    #[cfg(feature="containers")]
    fn system_deps(&mut self, pkg: packages::Package, ctx: &mut Context)
        -> Option<Vec<&'static str>>
    {
        match pkg {
            packages::BuildEssential => Some(vec!()),
            packages::Https => Some(vec!()),
            // Python
            packages::Python2 => Some(vec!("python")),
            packages::Python2Dev => Some(vec!()),
            packages::Python3 => Some(vec!("python3")),
            packages::Python3Dev => Some(vec!()),
            packages::PipPy2 => None,
            packages::PipPy3 => None,
            // Node.js
            packages::NodeJs if self.needs_node_legacy() => {
                match self.get_codename(ctx).expect("codename is set") {
                    | "precise"
                    | "trusty"
                    | "xenial"
                    => Some(vec!["nodejs", "nodejs-legacy"]),
                    // all newer releases
                    _ => Some(vec!["nodejs", "npm"]),
                }
            }
            packages::NodeJs => Some(vec!("nodejs")),
            packages::NodeJsDev => Some(vec!()),
            packages::Npm => Some(vec!()),
            packages::Yarn => None,
            // PHP
            packages::Php if self.has_php7() => {
                // In ubuntu xenial, php package does not bundles the json and zip modules required
                // by Composer
                Some(vec!("php-common", "php-cli", "php-json", "php-zip"))
            }
            packages::Php => Some(vec!("php5-common", "php5-cli")),
            packages::PhpDev => Some(vec!()),
            packages::Composer => None,
            // Ruby
            packages::Ruby if self.needs_rubygems() => {
                Some(vec!("ruby", "rubygems"))
            }
            packages::Ruby => Some(vec!("ruby")),
            packages::RubyDev => Some(vec!()),
            packages::Bundler => None,
            // VCS
            packages::Git => Some(vec!()),
            packages::Mercurial => Some(vec!()),
        }
    }

    #[cfg(feature="containers")]
    fn build_deps(&self, pkg: packages::Package) -> Option<Vec<&'static str>> {
        match pkg {
            packages::BuildEssential => Some(vec![
                "build-essential",
                "ca-certificates",
            ]),
            packages::Https => Some(vec!("ca-certificates")),
            // Python
            packages::Python2 => Some(vec!()),
            packages::Python2Dev => Some(vec!("python-dev")),
            packages::Python3 => Some(vec!()),
            packages::Python3Dev => Some(vec!("python3-dev")),
            packages::PipPy2 => None,
            packages::PipPy3 => None,
            // Node.js
            packages::NodeJs => Some(vec!()),
            packages::NodeJsDev => Some(vec!("nodejs-dev")),
            packages::Npm => Some(vec!("npm")),
            packages::Yarn => None,
            // PHP
            packages::Php => Some(vec!()),
            packages::PhpDev if self.has_php7() => Some(vec!("php-dev")),
            packages::PhpDev => Some(vec!("php5-dev")),
            packages::Composer => None,
            // Ruby
            packages::Ruby => Some(vec!()),
            packages::RubyDev => Some(vec!("ruby-dev")),
            packages::Bundler => None,
            // VCS
            packages::Git => Some(vec!("git")),
            packages::Mercurial => Some(vec!("mercurial")),
        }
    }

    #[cfg(feature="containers")]
    fn copy_apt_lists_from_cache(&self) -> io::Result<()> {
        let dir = format!("/vagga/cache/apt-lists-{}",
                          self.codename.as_ref().unwrap());
        let cache_dir = Path::new(&dir);
        if !cache_dir.exists() {
            return Ok(());
        }
        let dir = Path::new("/vagga/root/var/lib/apt/lists");
        Dir::new(dir).recursive(true).create()?;
        ScanDir::files().read(&cache_dir, |iter| {
            for (entry, name) in iter {
                FileCopy::new(&entry.path(), &dir.join(&name))
                    .atomic(true)
                    .time(CopyTimePolicy::Preserve)
                    .copy()?;
            }
            Ok(())
        }).map_err(|x| io::Error::new(io::ErrorKind::Other, x)).and_then(|x| x)
    }

    #[cfg(feature="containers")]
    fn copy_apt_lists_to_cache(&self) -> io::Result<()> {
        if !self.has_indices {
            return Ok(());
        }
        let dir = format!("/vagga/cache/apt-lists-{}",
            self.codename.as_ref().unwrap());
        let cache_dir = Path::new(&dir);
        Dir::new(&cache_dir).create()?;
        ScanDir::files().read("/vagga/root/var/lib/apt/lists", |iter| {
            for (entry, name) in iter {
                if name == "lock" { continue };
                FileCopy::new(&entry.path(), &cache_dir.join(name))
                    .atomic(true)
                    .time(CopyTimePolicy::Preserve)
                    .copy()?;
            }
            Ok(())
        }).map_err(|x| io::Error::new(io::ErrorKind::Other, x)).and_then(|x| x)
    }

    #[cfg(feature="containers")]
    fn copy_apt_archives_to_cache(&self, diff_dir: &Path, cache_dir: &Path) -> io::Result<()> {
        let cache_dir = cache_dir.join("archives");
        Dir::new(&cache_dir).create()?;

        let archives_dir = diff_dir.join("archives");
        if !archives_dir.exists() {
            return Ok(());
        }
        ScanDir::files().read(&archives_dir, |iter| {
            for (entry, name) in iter {
                if !name.ends_with(".deb") { continue }
                if !entry.file_type()?.is_file() { continue }

                let dst = cache_dir.join(name);
                if !dst.exists() {
                    FileCopy::new(&entry.path(), &dst)
                        .atomic(true)
                        .time(CopyTimePolicy::Preserve)
                        .copy()?;
                }
            }
            Ok(())
        })
            .map_err(|x| io::Error::new(io::ErrorKind::Other, x))
            .and_then(|x| x)?;

        remove_dir_all(diff_dir)
    }

    #[cfg(feature="containers")]
    fn ensure_basic_packages(&mut self, ctx: &mut Context, extra_packages: &[&str])
        -> Result<(), StepError>
    {
        if self.eatmydata == EatMyData::Need {
            self.ensure_codename(ctx)?;
            let eatmy = EMDParams::new(
                self.codename.as_ref().unwrap(), &self.config.arch
            );
            if let Some(params) = eatmy {
                if params.needs_universe {
                    debug!("Add Universe for eat my data");
                    self.enable_universe()?;
                    self.add_universe(ctx)?;
                }

                apt_get_update::<&str>(ctx, &[], &mut self.apt_update)?;

                match apt_get_install(ctx, &[params.package], &EatMyData::No) {
                    Ok(()) => self.eatmydata = params.find(ctx),
                    Err(e) => {
                        warn!(
                            "Could not install {} package. The build may be slower than usual. \
                            Cause: {}",
                            params.package, e
                        );
                    }
                }
            } else {
                info!("Unsupported distribution for eatmydata. Ignoring");
                self.eatmydata = EatMyData::No;
            }
        }

        if !extra_packages.is_empty() {
            apt_get_update::<&str>(ctx, &[], &mut self.apt_update)?;
            apt_get_install(ctx, &extra_packages, &self.eatmydata)?;
        }

        Ok(())
    }
}

impl BuildStep for Ubuntu {
    fn name(&self) -> &'static str { "Ubuntu" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("codename", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(guard, UbuntuRelease {
            codename: Some(self.0.clone()),
            version: None,
            url: None,
            arch: String::from("amd64"),  // TODO(tailhook) detect
            keep_chfn_command: false,
            eatmydata: true,
        })?;
        if build {
            guard.distro.bootstrap(&mut guard.ctx)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for UbuntuUniverse {
    fn name(&self) -> &'static str { "UbuntuUniverse" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, _hash: &mut Digest)
        -> Result<(), VersionError>
    {
        // Nothing to do: singleton command
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        let ref mut ctx = guard.ctx;
        guard.distro.specific(|u: &mut Distro| {
            u.enable_universe()?;
            if build {
                u.add_universe(ctx)?;
            }
            Ok(())
        })
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for UbuntuPPA {
    fn name(&self) -> &'static str { "UbuntuPPA" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("ppa_name", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let ref mut ctx = guard.ctx;
            guard.distro.specific(|u: &mut Distro| {
                // Need to install eatmydata before installing https
                // transport because latter takes ~ 100 seconds without
                // libeatmydata
                u.ensure_basic_packages(ctx, &[])?;
                u.add_ubuntu_ppa(ctx, &self.0)
            })?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for AptTrust {
    fn name(&self) -> &'static str { "AptTrust" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.opt_field("server", &self.server);
        hash.field("keys", &self.keys);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let ref mut ctx = guard.ctx;
            guard.distro.specific(|u: &mut Distro| {
                u.add_apt_key(ctx, &self)
            })?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for UbuntuRepo {
    fn name(&self) -> &'static str { "UbuntuRepo" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.opt_field("url", &self.url);
        hash.opt_field("suite", &self.suite);
        hash.field("components", &self.components);
        hash.field("trusted", self.trusted);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if self.url.as_ref().map_or(false, |url| url.starts_with("https:")) {
            guard.distro.specific(|u: &mut Distro| {
                if u.apt_https == AptHttps::No {
                    u.apt_https = AptHttps::Need;
                }
                Ok(())
            })?;
        }
        if build {
            let ref mut ctx = guard.ctx;
            guard.distro.specific(|u: &mut Distro| {
                // Need to install eatmydata before installing https
                // transport because latter takes ~ 100 seconds without
                // libeatmydata
                u.ensure_basic_packages(ctx, &[])?;
                u.add_debian_repo(ctx, &self)?;
                Ok(())
            })?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for UbuntuRelease {
    fn name(&self) -> &'static str { "UbuntuRelease" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.opt_field("codename", &self.codename);
        hash.opt_field("version", &self.version);
        hash.opt_field("url", &self.url);
        hash.field("arch", &self.arch);
        hash.field("keep_chfn_command", self.keep_chfn_command);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(guard, self.clone())?;
        if build {
            guard.distro.bootstrap(&mut guard.ctx)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

#[cfg(feature="containers")]
mod build {
    use std::io::BufRead;
    use std::os::unix::fs::PermissionsExt;

    use unshare::Command;
    use crate::builder::commands::generic::capture_command;

    use crate::capsule::download::download_file;
    use crate::process_util::CaptureOutput;

    use super::*;

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

    pub fn fetch_ubuntu_core(ctx: &mut Context, rel: &UbuntuRelease)
        -> Result<(), String>
    {
        let urls = if let Some(ref url) = rel.url {
            vec![url.to_string()]
        } else if let Some(ref version) = rel.version {
            let ver = if version.len() > 5 && version[5..].starts_with('.') {
                // ignore everything after second dot
                // i.e. 12.04.5 == 12.04
                &version[..5]
            } else {
                &version[..]
            };
            let codename = match ver {
                "12.04" => "precise",
                "14.04" => "trusty",
                "14.10" => "utopic",
                "15.04" => "vivid",
                "15.10" => "wily",
                "16.04" => "xenial",
                // Note: no new names here
                // This list is only provided for backwards compatibility
                _ => return Err(format!("Unknown version {:?}. \
                    Note, we only have certain number of hardcoded versions \
                    for backwards-compatibility. You should use `codename` \
                    property (or `!Ubuntu` step) for first-class support",
                   version)),
            };
            warn!("Note `!UbuntuRelease {{ version: {0:?} }}` is deprecated. \
                   Use `!UbuntuRelease {{ codename: {1:?} }}` or `!Ubuntu {1:?}` \
                   instead", version, codename);
            vec![
                format!(
                    "https://partner-images.canonical.com/core/\
                     {codename}/current/\
                     ubuntu-{codename}-core-cloudimg-{arch}-root.tar.gz",
                    arch=rel.arch, codename=codename),
                format!(
                    "https://partner-images.canonical.com/core/unsupported/\
                     {codename}/current/\
                     ubuntu-{codename}-core-cloudimg-{arch}-root.tar.gz",
                    arch=rel.arch, codename=codename),
            ]
        } else if let Some(ref codename) = rel.codename {
            vec![
                format!(
                    "https://partner-images.canonical.com/core/\
                     {codename}/current/\
                     ubuntu-{codename}-core-cloudimg-{arch}-root.tar.gz",
                    arch=rel.arch, codename=codename),
                format!(
                    "https://partner-images.canonical.com/core/unsupported/\
                     {codename}/current/\
                     ubuntu-{codename}-core-cloudimg-{arch}-root.tar.gz",
                    arch=rel.arch, codename=codename),
            ]
        } else {
            return Err(format!("UbuntuRelease tag must contain one of \
                `codename` (preferred), `version` (deprecated) \
                or `url` (if you need something special)"));
        };
        let filename = download_file(&mut ctx.capsule, &urls, None, false)?;
        unpack_file(ctx, &filename, &Path::new("/vagga/root"), &[],
            &[Path::new("dev"),
              Path::new("sys"),
              Path::new("proc"),
              Path::new("run"),
              Path::new("etc/resolv.conf"),
              Path::new("etc/hosts")], false)?;

        Ok(())
    }

    pub fn set_sources_list(ctx: &mut Context, distro: &mut Distro)
        -> Result<(), String>
    {
        let mirror = ctx.settings.ubuntu_mirror().to_string();
        let suite = distro.get_codename(ctx)?;
        let sources_list = Path::new("/vagga/root/etc/apt/sources.list");
        let sources_list_tmp = Path::new("/vagga/root/etc/apt/sources.list.tmp");
        File::create(sources_list_tmp)
            .and_then(|mut f| {
                write!(
                    &mut f,
                    "deb {mirror} {suite} main restricted\n\
                     deb {mirror} {suite}-updates main restricted\n\
                     deb {mirror} {suite}-security main restricted\n",
                    mirror=mirror, suite=suite
                )
            })
            .map_err(|e| format!("Error writing sources.list file: {}", e))?;
        rename(sources_list_tmp, sources_list)
            .map_err(|e| format!("Error renaming sources.list file: {}", e))
    }

    pub fn init_ubuntu_core(ctx: &mut Context, distro: &mut Distro)
        -> Result<(), String>
    {
        // Do not attempt to start init scripts
        let policy_file = Path::new("/vagga/root/usr/sbin/policy-rc.d");
        File::create(&policy_file)
            .and_then(|mut f| f.write_all(b"#!/bin/sh\nexit 101\n"))
            .map_err(|e| format!("Error writing policy-rc.d file: {}", e))?;
        set_permissions(&policy_file, Permissions::from_mode(0o755))
            .map_err(|e| format!("Can't chmod file: {}", e))?;

        // Do not need to fsync() after package installation
        File::create(
                &Path::new("/vagga/root/etc/dpkg/dpkg.cfg.d/02apt-speedup"))
            .and_then(|mut f| f.write_all(b"force-unsafe-io"))
            .map_err(|e| format!("Error writing dpkg config: {}", e))?;

        // Do not install recommends by default
        File::create(
                &Path::new("/vagga/root/etc/apt/apt.conf.d/01norecommend"))
            .and_then(|mut f| f.write_all(br#"
                APT::Install-Recommends "0";
                APT::Install-Suggests "0";
            "#))
            .map_err(|e| format!("Error writing apt config: {}", e))?;

        revert_name_files()?;

        set_sources_list(ctx, distro)?;

        if find_cmd(ctx, "locale-gen").is_err() {
            distro.ensure_basic_packages(ctx, &["locales"])?;
        }

        let mut cmd = command(ctx, "locale-gen")?;
        cmd.arg("en_US.UTF-8");
        run(cmd)?;

        // TODO(tailhook) reconsider this. It was fun to remove unneeded files
        //                until we have !Container which fails ot reuse ubuntu
        //                container when /var/lib/apt is clean
        // ctx.add_remove_dir(Path::new("/var/lib/apt"));
        // TODO(tailhook) decide if we want to delete package databases
        // ctx.add_remove_dir(Path::new("/var/lib/dpkg"));
        return Ok(());
    }

    pub fn clobber_chfn() -> Result<(), String> {
        try_msg!(symlink("/bin/true", "/vagga/root/usr/bin/.tmp.chfn"),
            "Can't clobber chfn (symlink error): {err}");
        try_msg!(rename("/vagga/root/usr/bin/.tmp.chfn",
                        "/vagga/root/usr/bin/chfn"),
            "Can't clobber chfn (rename error): {err}");
        Ok(())
    }

    pub fn configure(guard: &mut Guard, config: UbuntuRelease)
        -> Result<(), StepError>
    {
        guard.distro.set(Distro {
            eatmydata: if config.eatmydata { EatMyData::Need } else { EatMyData::No },
            config,
            codename: None, // unknown yet
            apt_update: true,
            apt_https: AptHttps::No,
            apt_hkps: AptHkps::No,
            has_indices: false,
            has_universe: false,
        })?;
        configure_common(&mut guard.ctx)
    }

    pub fn configure_common(ctx: &mut Context) -> Result<(), StepError> {
        init_apt_cache(ctx)?;
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

    fn init_apt_cache(ctx: &mut Context) -> Result<(), StepError> {
        // TODO: Fallback only on mount error
        if ctx.add_cache_dir_overlay(&APT_CACHE, APT_CACHE_NAME).is_err() {
            ctx.add_cache_dir(&APT_CACHE, APT_CACHE_NAME)
                .map_err(|e| format!("Error mounting apt cache: {}", e))?;
        }
        Ok(())
    }

    pub fn apt_get_update<T: AsRef<OsStr>>(ctx: &mut Context, options: &[T],
        needed: &mut bool)
        -> Result<(), StepError>
    {
        if !*needed {
            return Ok(());
        }
        *needed = false;
        let mut cmd = command(ctx, "apt-get")?;
        cmd.arg("-oDir::cache::pkgcache=");
        cmd.arg("-oDir::cache::srcpkgcache=");
        cmd.arg("update");
        cmd.args(options);
        run(cmd)
             .map_err(|error| {
                 if ctx.settings.ubuntu_mirror.is_none() {
                     warn!("The `apt-get update` failed. You have no mirror \
                         setup, and default one is not always perfect.\n\
                         Add the following to your ~/.vagga.yaml:\
                         \n  ubuntu-mirror: http://CC.archive.ubuntu.com/ubuntu\n\
                         Where CC is a two-letter country code where you currently are.\
                         ");
                 } else {
                     warn!("The `apt-get update` failed. \
                         If this happens too often, consider changing \
                         the `ubuntu-mirror` in settings");
                 }
                 error
             })
    }

    pub fn apt_get_lock(enabled: bool) -> Result<Option<Lock>, StepError> {
        if enabled {
            Lock::exclusive_wait(
                Path::new("/vagga/root/var/cache/apt/apt-get-install.lock"),
                false,
                "Another build process is executing `apt-get install` command \
                 against the same apt cache. Waiting ...")
            .map(Some)
            .map_err(|e| StepError::Lock(
                "Cannot aquire lock before running `apt-get install`", e))
        } else {
            Ok(None)
        }
    }

    pub(in super) fn eat_my_data(cmd: &mut Command, ctx: &Context, emd: &EatMyData) {
        if let EatMyData::Preload(preload) = emd {
            match ctx.environ.get("LD_PRELOAD") {
                None => {
                    cmd.env("LD_PRELOAD", preload);
                },
                Some(v) => {
                    if !v.is_empty() {
                        cmd.env("LD_PRELOAD", format!("{}:{}", v, preload));
                    } else {
                        cmd.env("LD_PRELOAD", preload);
                    }
                },
            }
        }
    }

    pub(in super) fn apt_get_install<T: AsRef<OsStr>>(ctx: &mut Context,
        packages: &[T], emd: &EatMyData)
        -> Result<(), StepError>
    {
        let mut cmd = command(ctx, "apt-get")?;
        eat_my_data(&mut cmd, ctx, emd);
        cmd.arg("-oDir::cache::pkgcache=");
        cmd.arg("-oDir::cache::srcpkgcache=");
        if ctx.settings.ubuntu_skip_locking {
            cmd.arg("-oDebug::NoLocking=yes");
        }
        cmd.arg("install");
        cmd.arg("-y");
        cmd.args(packages);

        let _lock = apt_get_lock(
            !ctx.settings.ubuntu_skip_locking &&
            !ctx.is_cached_dir_overlay(&APT_CACHE)
        )?;
        run(cmd)
    }


    impl EMDParams {
        pub fn new(codename: &str, arch: &str) -> Option<EMDParams> {
            match (codename, arch) {
                ("trusty", _) |
                ("precise", _) => Some(EMDParams {
                    needs_universe: true,
                    package: "eatmydata",
                }),
                _ => Some(EMDParams {
                    needs_universe: false,
                    package: "libeatmydata1"
                }),
            }
        }

        pub fn find(&self, ctx: &mut Context) -> EatMyData {
            self._find(ctx)
                .map_or_else(
                    |e| {
                        warn!("{}", e);
                        EatMyData::No
                    },
                    |p| {
                        info!("Eatmydata activated: {}", p);
                        EatMyData::Preload(p)
                    }
                )
        }

        pub fn _find(&self, ctx: &mut Context) -> Result<String, String> {
            let emd_pkg_output = capture_command(
                ctx,
                &["/usr/bin/dpkg-query".to_string(), "-L".to_string(), self.package.to_string()],
                &[],
                CaptureOutput::Stdout
            )
                .map_err(|e| format!("Error fetching {} package content: {}", self.package, e))?;
            let emd_pkg_output = String::from_utf8(emd_pkg_output)
                .map_err(|e| format!("Cannot decode dpkg-query output to utf-8: {}", e))?;

            for emd_pkg_content_file in emd_pkg_output.lines() {
                let emd_pkg_content_file = emd_pkg_content_file.trim();
                if emd_pkg_content_file.ends_with("/libeatmydata.so") {
                    return Ok(emd_pkg_content_file.to_string());
                }
            }

            Err(format!("Could not find libeatmydata.so dynamic library"))
        }
    }
}
