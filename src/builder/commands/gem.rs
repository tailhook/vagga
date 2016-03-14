use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write};

use regex::Regex;

use super::super::context::Context;
use super::super::packages;
use super::generic::capture_command;
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use config::builders::{GemSettings, GemBundleInfo};
use process_util::capture_stdout;

const DEFAULT_GEM_EXE: &'static str = "gem";
const BIN_DIR: &'static str = "/usr/local/bin";
const RUBY_VERSION_WITH_GEM: f32 = 1.9;
const GEM_VERSION_WITH_NO_DOCUMENT_OPT: f32 = 2.0;


impl Default for GemSettings {
    fn default() -> Self {
        GemSettings {
            install_ruby: true,
            gem_exe: None,
        }
    }
}

fn ruby_version(ctx: &mut Context) -> Result<f32, String> {
    let args = [
        "ruby".to_owned(),
        "--version".to_owned(),
    ];

    let ruby_ver = try!(capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error parsing ruby version: {}", e)))
        .map_err(|e| format!("Error getting ruby version: {}", e)));

    let re = Regex::new(r#"^ruby (\d+?\.\d+?)\."#).expect("Invalid regex");
    let version = try!(re.captures(&ruby_ver)
        .and_then(|cap| cap.at(1))
        .ok_or("Ruby version was not found".to_owned()));

    version.parse::<f32>()
        .map_err(|e| format!("Erro parsing ruby version: {}", e))
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

fn scan_features(settings: &GemSettings, git_required: bool)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);

    if settings.install_ruby {
        res.push(packages::Ruby);
        res.push(packages::RubyDev);
    }

    res.push(packages::Bundler);

    if git_required {
        res.push(packages::Git);
    }

    res
}

pub fn install(distro: &mut Box<Distribution>,
    ctx: &mut Context, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.gem_settings, false);
    try!(packages::ensure_packages(distro, ctx, &features));

    if try!(ruby_version(ctx)) < RUBY_VERSION_WITH_GEM {
        try!(packages::ensure_packages(distro, ctx, &[packages::RubyGems]));
    }

    try!(configure(ctx));

    if pkgs.len() == 0 {
        return Ok(());
    }

    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = try!(command(ctx, &gem_exe));
    cmd.arg("install");
    cmd.args(&["--bindir", BIN_DIR]);

    if try!(gem_version(ctx)) < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
        cmd.args(&["--no-rdoc", "--no-ri"]);
    } else {
        cmd.arg("--no-document");
    }

    cmd.args(pkgs);
    try!(run(cmd));
    Ok(())
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

pub fn bundle(distro: &mut Box<Distribution>,
    ctx: &mut Context, info: &GemBundleInfo)
    -> Result<(), StepError>
{
    let git_required = try!(requires_git(&info.gemfile));
    let features = scan_features(&ctx.gem_settings, git_required);
    try!(packages::ensure_packages(distro, ctx, &features));

    let version = try!(ruby_version(ctx));
    if version < RUBY_VERSION_WITH_GEM {
        try!(packages::ensure_packages(distro, ctx, &[packages::RubyGems]));
    }

    try!(configure(ctx));

    let mut cmd = try!(command(ctx, "bundle"));
    cmd.args(&["install", "--system", "--binstubs", BIN_DIR]);

    cmd.arg("--gemfile");
    cmd.arg(&info.gemfile);

    if !info.without.is_empty() {
        cmd.arg("--without");
        cmd.args(&info.without);
    }

    run(cmd)
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
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

    if try!(gem_version(ctx)) < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
        cmd.args(&["--no-rdoc", "--no-ri"]);
    } else {
        cmd.arg("--no-document");
    }

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
