use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write, BufReader, BufRead};
use std::os::unix::ffi::OsStrExt;

use regex::Regex;
use quire::validate as V;

use super::super::context::Context;
use super::super::packages;
use super::generic::{capture_command, run_command_at_env};
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use process_util::capture_stdout;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

const DEFAULT_GEM_EXE: &'static str = "/usr/bin/gem";
const BIN_DIR: &'static str = "/usr/local/bin";
const GEM_VERSION_WITH_NO_DOCUMENT_OPT: f32 = 2.0;
const VALID_TRUST_POLICIES: [&'static str; 3] = ["LowSecurity", "MediumSecurity", "HighSecurity"];


#[derive(Debug)]
pub struct GemInstall(Vec<String>);
tuple_struct_decode!(GemInstall);

impl GemInstall {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct GemConfig {
    pub install_ruby: bool,
    pub gem_exe: Option<String>,
    pub update_gem: bool,
}

impl GemConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("install_ruby", V::Scalar::new().default(true))
        .member("gem_exe", V::Scalar::new().optional())
        .member("update_gem", V::Scalar::new().default(true))
    }
}

#[derive(RustcDecodable, Debug)]
pub struct GemBundle {
    pub gemfile: PathBuf,
    pub without: Vec<String>,
    pub trust_policy: Option<String>,
}

impl GemBundle {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("gemfile", V::Scalar::new().default("Gemfile"))
        .member("without", V::Sequence::new(V::Scalar::new()))
        .member("trust_policy", V::Scalar::new().optional())
    }
}

impl Default for GemConfig {
    fn default() -> Self {
        GemConfig {
            install_ruby: true,
            gem_exe: None,
            update_gem: true,
        }
    }
}

fn no_doc_args(ctx: &mut Context) -> Result<Vec<&'static str>, String> {
    if ctx.gem_settings.update_gem {
        Ok(vec!("--no-document"))
    } else {
        let version = try!(gem_version(ctx));
        if version < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
            Ok(vec!("--no-rdoc", "--no-ri"))
        } else {
            Ok(vec!("--no-document"))
        }
    }
}

fn gem_version(ctx: &mut Context) -> Result<f32, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        gem_exe,
        "--version".to_owned(),
    ];

    let gem_ver = try!(capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error parsing gem version: {}", e)))
        .map_err(|e| format!("Error getting gem version: {}", e)));

    let re = Regex::new(r#"^(\d+?\.\d+?)\."#).expect("Invalid regex");
    let version = try!(re.captures(&gem_ver)
        .and_then(|cap| cap.at(1))
        .ok_or("Gem version was not found".to_owned()));

    version.parse::<f32>()
        .map_err(|e| format!("Erro parsing gem version: {}", e))
}

fn gem_cache_dir(ctx: &mut Context) -> Result<PathBuf, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        gem_exe,
        "env".to_owned(),
        "gemdir".to_owned(),
    ];

    let gem_dir = try!(capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error getting gem dir: {}", e))));

    Ok(Path::new(gem_dir.trim()).join("cache"))
}

fn requires_git(gemfile: &Path) -> Result<bool, String> {
    let gemfile = Path::new("/work").join(gemfile);

    let re = Regex::new(
        r#"(git .*? do)|(:(git|github|gist|bitbucket) =>)|(git_source\(.*?\))"#
    ).expect("Invalid regex");

    let gemfile_data = {
        let mut buf = String::new();
        try!(File::open(&gemfile)
            .and_then(|mut f| f.read_to_string(&mut buf))
            .map_err(|e| format!("Error reading Gemfile ({:?}): {}", &gemfile, e)));

        buf
    };

    Ok(re.is_match(&gemfile_data))
}

fn scan_features(settings: &GemConfig, info: Option<&GemBundle>)
    -> Result<Vec<packages::Package>, String>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);

    if settings.install_ruby {
        res.push(packages::Ruby);
        res.push(packages::RubyDev);
    }

    res.push(packages::Https);
    res.push(packages::Bundler);

    if let Some(info) = info {
        let git_required = try!(requires_git(&info.gemfile));
        if git_required {
            res.push(packages::Git);
        }
    }

    Ok(res)
}

pub fn install(distro: &mut Box<Distribution>,
    ctx: &mut Context, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = try!(scan_features(&ctx.gem_settings, None));
    try!(packages::ensure_packages(distro, ctx, &features));

    try!(configure(ctx));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = try!(command(ctx, &gem_exe));
    cmd.arg("install");
    cmd.args(&["--bindir", BIN_DIR]);

    let no_doc = try!(no_doc_args(ctx));
    cmd.args(&no_doc);

    cmd.args(pkgs);
    try!(run(cmd));
    Ok(())
}

pub fn bundle(distro: &mut Box<Distribution>,
    ctx: &mut Context, info: &GemBundle)
    -> Result<(), StepError>
{
    let features = try!(scan_features(&ctx.gem_settings, Some(info)));
    try!(packages::ensure_packages(distro, ctx, &features));

    try!(configure(ctx));

    let mut cmd = try!(command(ctx, "bundle"));
    cmd.args(&["install", "--system", "--binstubs", BIN_DIR]);

    cmd.arg("--gemfile");
    cmd.arg(&info.gemfile);

    if !info.without.is_empty() {
        cmd.arg("--without");
        cmd.args(&info.without);
    }

    if let Some(ref trust_policy) = info.trust_policy {
        if !VALID_TRUST_POLICIES.contains(&trust_policy.as_ref()) {
            return return Err(From::from(format!(
                "Value of 'GemBundle.trust_policy' must be \
                    '{}', '{}' or '{}', '{}' given",
                VALID_TRUST_POLICIES[0],
                VALID_TRUST_POLICIES[1],
                VALID_TRUST_POLICIES[2],
                trust_policy
            )))
        }
        cmd.arg("--trust-policy");
        cmd.arg(trust_policy);
    }

    run(cmd)
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
    if ctx.gem_settings.gem_exe.is_none() &&
        ctx.gem_settings.update_gem
    {
        let mut args = vec!(
            DEFAULT_GEM_EXE.to_owned(),
            "update".to_owned(),
            "--system".to_owned(),
        );

        let version = try!(gem_version(ctx));
        if version < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
            args.extend(vec!("--no-rdoc".to_owned(), "--no-ri".to_owned()));
        } else {
            args.push("--no-document".to_owned());
        }

        // Debian based distros doesn't allow updating gem unless this flag is set
        let env = [("REALLY_GEM_UPDATE_SYSTEM", "1")];
        try!(run_command_at_env(ctx, &args, Path::new("/work"), &env));
    }

    let gem_cache = try!(gem_cache_dir(ctx));
    try!(ctx.add_cache_dir(&gem_cache,
                           "gems-cache".to_string()));

    Ok(())
}

pub fn setup_bundler(ctx: &mut Context) -> Result<(), String> {
    try!(configure(ctx));

    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = try!(command(ctx, gem_exe));
    cmd.args(&["install", "bundler"]);

    let no_doc = try!(no_doc_args(ctx));
    cmd.args(&no_doc);

    try!(run(cmd));

    Ok(())
}

pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = try!(command(ctx, gem_exe));
    cmd.arg("list");
    cmd.arg("--local");

    try!(capture_stdout(cmd)
        .and_then(|out| {
            File::create("/vagga/container/gems-list.txt")
            .and_then(|mut f| f.write_all(&out))
            .map_err(|e| format!("Error dumping gems package list: {}", e))
        }));
    Ok(())
}

impl BuildStep for GemInstall {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("GemInstall", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(install(&mut guard.distro, &mut guard.ctx, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for GemConfig {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.bool("install_ruby", self.install_ruby);
        hash.opt_field("gem_exe", &self.gem_exe);
        hash.bool("update_gem", self.update_gem);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        guard.ctx.gem_settings = self.clone();
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for GemBundle {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = Path::new("/work").join(&self.gemfile);

        hash.item(&self.gemfile.as_os_str().as_bytes());
        let gemlock = try!(path.parent()
            .map(|dir| dir.join("Gemfile.lock"))
            .ok_or("Gemfile should be under /work".to_owned()));
        if gemlock.exists() {
            let mut lockfile = try!(File::open(&path)
                .map_err(|e| VersionError::Io(e, gemlock.clone())));
            try!(hash.stream(&mut lockfile)
                .map_err(|e| VersionError::Io(e, gemlock.clone())));
        }

        let f = try!(File::open(&path)
            .map_err(|e| VersionError::Io(e, path.clone())));
        let reader = BufReader::new(f);

        for line in reader.lines() {
            let line = try!(line
                .map_err(|e| VersionError::Io(e, path.clone())));
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue
            }
            hash.item(line);
        }

        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            try!(bundle(&mut guard.distro, &mut guard.ctx, self));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
