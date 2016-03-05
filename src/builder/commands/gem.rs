use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

use regex::Regex;

use super::super::context::{Context};
use super::super::packages;
use super::generic::{run_command, capture_command};
use builder::error::StepError;
use builder::distrib::Distribution;
use builder::commands::generic::{command, run};
use config::builders::{GemSettings, GemBundleInfo};
use process_util::capture_stdout;

const DEFAULT_GEM_EXE: &'static str = "gem";

const GEM_HOME: &'static str = "/usr/local/lib/rubygems";
const GEM_CACHE: &'static str = "/usr/local/lib/rubygems/cache";
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
        "/bin/sh".to_owned(),
        "-exc".to_owned(),
        "ruby --version | cut -d ' ' -f 2 | cut -d '.' -f -2".to_owned(),
    ];

    capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error getting ruby version: {}", e))
            .and_then(|v| v.trim().parse::<f32>()
                .map_err(|e| format!("Erro parsing ruby version ({}): {}", &v, e))
            )
        )
}

fn gem_version(ctx: &mut Context) -> Result<f32, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        "/bin/sh".to_owned(),
        "-exc".to_owned(),
        format!("{} --version | cut -d '.' -f -2", gem_exe),
    ];

    capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error getting gem version: {}", e))
            .and_then(|v| v.trim().parse::<f32>()
                .map_err(|e| format!("Erro parsing gem version ({}): {}", &v, e))
            )
        )
}
fn gem_dir(ctx: &mut Context) -> Result<String, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        gem_exe,
        "env".to_owned(),
        // "find /usr/lib/ruby/gems -name 'cache' | grep -E /usr/lib/ruby/.*?/cache".to_owned(),
        "gemdir".to_owned(),
    ];

    capture_command(ctx, &args, &[])
        .and_then(|x| String::from_utf8(x)
            .map_err(|e| format!("Error getting gem dir: {}", e)))
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
    let re = try!(
        Regex::new(r#"(git .*? do)|(:(git|github|gist|bitbucket) =>)|(git_source\(.*?\))"#)
        .map_err(|e| format!("Regex error: {}", e))
    );

    let gemfile_data = {
        let mut buf = String::new();
        try!(File::open(gemfile)
            .and_then(|mut f| f.read_to_string(&mut buf))
            .map_err(|e| format!("Error reading Gemfile: {}", e)));

        buf
    };

    Ok(re.is_match(&gemfile_data))
}

pub fn bundle(distro: &mut Box<Distribution>,
    ctx: &mut Context, info: &GemBundleInfo)
    -> Result<(), StepError>
{
    let gemfile = info.gemfile.clone()
        .unwrap_or(Path::new("/work").join("Gemfile"));

    let git_required = try!(requires_git(&gemfile));
    let features = scan_features(&ctx.gem_settings, git_required);
    try!(packages::ensure_packages(distro, ctx, &features));

    let version = try!(ruby_version(ctx));
    if version < RUBY_VERSION_WITH_GEM {
        try!(packages::ensure_packages(distro, ctx, &[packages::RubyGems]));
    }

    let mut cmd = try!(command(ctx, "bundle"));
    cmd.args(&["install", "--system", "--binstubs", BIN_DIR]);

    if let Some(ref gemfile) = info.gemfile {
        cmd.arg("--gemfile");
        cmd.arg(gemfile);
    }

    if !info.with.is_empty() {
        cmd.arg("--with");
        cmd.args(&info.with);
    }

    if !info.without.is_empty() {
        cmd.arg("--without");
        cmd.args(&info.without);
    }

    if info.deployment { cmd.arg("--deployment"); }

    run(cmd)
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
    try!(ctx.add_cache_dir(Path::new(GEM_CACHE),
                           "gems-cache".to_string()));
    ctx.environ.insert("GEM_HOME".to_owned(),
                       GEM_HOME.to_owned());

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
