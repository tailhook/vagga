use std::io::Write;
use std::fs::{File, OpenOptions};
use std::fmt::{Write as WriteFmt};
use std::path::Path;

use quire::validate as V;
use unshare::{Command, Stdio};
use rand::{thread_rng, Rng};
use regex::Regex;

use super::super::context::{Context};
use super::super::capsule;
use super::super::packages;
use file_util::Dir;
use process_util::capture_stdout;
use builder::distrib::{Distribution, Named, DistroBox};
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use config::version::Version;
use builder::dns::revert_name_files;


pub static LATEST_VERSION: &'static str = "v3.4";
static ALPINE_VERSION_REGEX: &'static str = r"^v\d+.\d+$";
static MIRRORS: &'static str = include_str!("../../../alpine/MIRRORS.txt");


const VERSION_WITH_PHP5: &'static str = "v3.4";


// Build Steps
#[derive(Debug)]
pub struct Alpine(String);
tuple_struct_decode!(Alpine);

#[derive(Debug, RustcDecodable)]
pub struct AlpineRepo {
    url: Option<String>,
    branch: Option<String>,
    repo: String,
    tag: Option<String>,
}

impl Alpine {
    pub fn config() -> V::Scalar {
        V::Scalar::new()
    }
}

impl AlpineRepo {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new().optional())
        .member("branch", V::Scalar::new().optional())
        .member("repo", V::Scalar::new())
        .member("tag", V::Scalar::new().optional())
    }
}

// Distro
#[derive(Debug)]
pub struct Distro {
    pub version: String,
    pub mirror: String,
    pub base_setup: bool,
    pub apk_update: bool,
}

impl Named for Distro {
    fn static_name() -> &'static str { "alpine" }
}

impl Distribution for Distro {
    fn name(&self) -> &'static str { "Alpine" }
    fn bootstrap(&mut self, ctx: &mut Context) -> Result<(), StepError> {
        if !self.base_setup {
            self.base_setup = true;
            setup_base(ctx, &self.version, &self.mirror)?;
            revert_name_files()?;
        }
        Ok(())
    }
    fn add_repo(&mut self, ctx: &mut Context, repo: &str)
        -> Result<(), StepError>
    {
        let repo_parts = repo.split('/').collect::<Vec<_>>();
        let (branch, repository) = match repo_parts.len() {
            1 => (None, repo_parts[0]),
            2 => (Some(repo_parts[0]), repo_parts[1]),
            _ => {
                return Err(StepError::from(format!(
                    "Cannot parse repository string. \
                     Should be in the next formats: \
                     'branch/repository' or 'repository'. \
                     But was: '{}'", repo)));
            },
        };
        let alpine_repo = AlpineRepo {
            url: Some(self.mirror.clone()),
            branch: branch.map(|x| x.to_string()),
            repo: repository.to_string(),
            tag: None,
        };
        self.add_alpine_repo(ctx, &alpine_repo)?;
        Ok(())
    }
    fn install(&mut self, ctx: &mut Context, pkgs: &[String])
        -> Result<(), StepError>
    {
        self.bootstrap(ctx)?;
        let mut apk_args = vec!();
        if self.apk_update {
            self.apk_update = false;
            apk_args.push("--update-cache");
        }
        apk_args.extend(&["--root", "/vagga/root"]);
        apk_args.push("add");
        capsule::apk_run(&apk_args, &pkgs[..])?;
        Ok(())
    }
    fn ensure_packages(&mut self, ctx: &mut Context,
        features: &[packages::Package])
        -> Result<Vec<packages::Package>, StepError>
    {
        self.bootstrap(ctx)?;
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
            capsule::apk_run(&[
                "--root", "/vagga/root",
                "add",
                ], &to_install[..])?;
        }
        return Ok(unsupp);
    }
    fn finish(&mut self, ctx: &mut Context) -> Result<(), String>
    {
        let pkgs = ctx.build_deps.clone().into_iter().collect();
        remove(ctx, &pkgs)?;
        let mut cmd = Command::new("/vagga/bin/apk");
        cmd
            .stdin(Stdio::null())
            .env_clear()
            .arg("--root").arg("/vagga/root")
            .arg("-vv")
            .arg("info");
        capture_stdout(cmd)
            .map_err(|e| format!("Error dumping package list: {}", e))
            .and_then(|out| {
                File::create("/vagga/container/alpine-packages.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            })?;
        Ok(())
    }
}

impl Distro {
    pub fn add_alpine_repo(&mut self, _: &mut Context, repo: &AlpineRepo)
        -> Result<(), String>
    {
        self.apk_update = true;

        let mut repo_line = String::new();
        if let Some(ref tag) = repo.tag {
            write!(&mut repo_line, "@{} ", tag).unwrap();
        }
        let url = repo.url.as_ref().unwrap_or(&self.mirror);
        let normalized_url = if !url.ends_with("/") {
            format!("{}/", url)
        } else {
            url.to_string()
        };
        write!(&mut repo_line, "{}", normalized_url).unwrap();
        write!(&mut repo_line, "{}/{}",
            &repo.branch.as_ref().unwrap_or(&self.version),
            &repo.repo).unwrap();

        OpenOptions::new().append(true)
            .open("/vagga/root/etc/apk/repositories")
            .and_then(|mut f| write!(&mut f, "{}\n", &repo_line))
            .map_err(|e| format!("Can't write repositories file: {}", e))?;

        Ok(())
    }

    fn php_build_deps(&self) -> Vec<&'static str> {
        let version_with_php5 = Version(VERSION_WITH_PHP5);
        let current_version = Version(self.version.as_ref());

        if current_version < version_with_php5 {
            vec!("php")
        } else {
            vec!("php5")
        }
    }

    fn php_system_deps(&self) -> Vec<&'static str> {
        let version_with_php5 = Version(VERSION_WITH_PHP5);
        let current_version = Version(self.version.as_ref());

        if current_version < version_with_php5 {
            vec!(
                "php", "php-cli", "php-openssl", "php-phar",
                "php-json", "php-pdo", "php-dom", "php-zip"
            )
        } else {
            vec!(
                "php5", "php5-cli", "php5-openssl", "php5-phar",
                "php5-json", "php5-pdo", "php5-dom", "php5-zip"
            )
        }
    }

    fn build_deps(&self, pkg: packages::Package) -> Option<Vec<&'static str>> {
        match pkg {
            packages::BuildEssential => Some(vec![
                "build-base",
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
            packages::Npm => Some(vec!()),
            // PHP
            packages::Php => Some(vec!()),
            packages::PhpDev => Some(self.php_build_deps()),
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

    fn system_deps(&self, pkg: packages::Package) -> Option<Vec<&'static str>> {
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
            packages::NodeJs => Some(vec!("nodejs")),
            packages::NodeJsDev => Some(vec!()),
            packages::Npm => Some(vec!("nodejs")),  // Need duplicate?
            // PHP
            packages::Php => Some(self.php_system_deps()),
            packages::PhpDev => Some(vec!()),
            packages::Composer => None,
            // Ruby
            packages::Ruby => Some(vec!("ruby", "ruby-io-console")),
            packages::RubyDev => Some(vec!()),
            packages::Bundler => None,
            // VCS
            packages::Git => Some(vec!()),
            packages::Mercurial => Some(vec!()),
        }
    }
}

pub fn choose_mirror() -> String {
    let repos = MIRRORS
        .split('\n')
        .map(|x| x.trim())
        .filter(|x| x.len() > 0 && !x.starts_with("#"))
        .collect::<Vec<&str>>();
    let mirror = thread_rng().choose(&repos)
        .expect("At least one mirror should work");
    debug!("Chosen mirror {}", mirror);
    return mirror.to_string();
}

fn check_version(version: &String) -> Result<(), String> {
    let version_regex = Regex::new(ALPINE_VERSION_REGEX)
                             .map_err(|e| format!("{}", e))?;
    match version_regex.is_match(&version) {
        true => Ok(()),
        false => Err(format!("Error checking alpine version: '{}'", version).to_string()),
    }
}

fn setup_base(ctx: &mut Context, version: &String, mirror: &String)
    -> Result<(), String>
{
    capsule::ensure_features(ctx, &[capsule::AlpineInstaller])?;
    check_version(version)?;
    try_msg!(Dir::new("/vagga/root/etc/apk").recursive(true).create(),
        "Error creating apk dir: {err}");
    if !Path::new("/vagga/root/etc/apk/repositories").exists() {
        File::create("/vagga/root/etc/apk/repositories")
            .and_then(|mut f| write!(&mut f, "{}{}/main\n",
                mirror, version))
            .map_err(|e| format!("Can't write repositories file: {}", e))?;
    }
    capsule::apk_run(&[
        "--update-cache",
        "--keys-dir=/etc/apk/keys",  // Use keys from capsule
        "--root=/vagga/root",
        "--initdb",
        "add",
        "alpine-base",
        ], &[])?;
    Ok(())
}


pub fn remove(_ctx: &mut Context, pkgs: &Vec<String>)
    -> Result<(), String>
{
    capsule::apk_run(&[
        "--root", "/vagga/root",
        "del",
        ], &pkgs[..])
}

pub fn configure(distro: &mut Box<Distribution>, ctx: &mut Context,
    ver: &str)
    -> Result<(), StepError>
{
    let mirror = ctx.settings.alpine_mirror.clone()
        .unwrap_or(choose_mirror());
    distro.set(Distro {
        version: ver.to_string(),
        mirror: mirror,
        base_setup: false,
        apk_update: true,
    })?;
    ctx.binary_ident = format!("{}-alpine-{}", ctx.binary_ident, ver);
    ctx.add_cache_dir(Path::new("/etc/apk/cache"),
                           "alpine-cache".to_string())?;
    ctx.environ.insert("LANG".to_string(),
                       "en_US.UTF-8".to_string());
    ctx.environ.insert("PATH".to_string(),
                       "/usr/local/sbin:/usr/local/bin:\
                        /usr/sbin:/usr/bin:/sbin:/bin\
                        ".to_string());
    Ok(())
}

impl BuildStep for Alpine {
    fn name(&self) -> &'static str { "Alpine" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("version", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(&mut guard.distro, &mut guard.ctx, &self.0)?;
        if build {
            guard.distro.bootstrap(&mut guard.ctx)?;
        } else {

        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for AlpineRepo {
    fn name(&self) -> &'static str { "AlpineRepo" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.opt_field("url", &self.url);
        hash.opt_field("branch", &self.branch);
        hash.field("repo", &self.repo);
        hash.opt_field("tag", &self.tag);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let ref mut ctx = guard.ctx;
            guard.distro.specific(|u: &mut Distro| {
                u.add_alpine_repo(ctx, &self)?;
                Ok(())
            })?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
