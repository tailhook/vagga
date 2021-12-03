use std::path::{Path, PathBuf};
use std::fs::{File, symlink_metadata};
use std::io::{Read, Write, BufReader, BufRead};

use regex::Regex;
use quire::validate as V;

#[cfg(feature="containers")]
use builder::context::Context;
#[cfg(feature="containers")]
use builder::packages;
#[cfg(feature="containers")]
use builder::commands::generic::{capture_command, run_command_at_env};
#[cfg(feature="containers")]
use builder::distrib::Distribution;
#[cfg(feature="containers")]
use builder::commands::generic::{command, run};
#[cfg(feature="containers")]
use process_util::capture_output;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

const DEFAULT_GEM_EXE: &'static str = "/usr/bin/gem";
const BIN_DIR: &'static str = "/usr/local/bin";
const GEM_VERSION_WITH_NO_DOCUMENT_OPT: f32 = 2.0;
const VALID_TRUST_POLICIES: [&'static str; 3] = ["LowSecurity", "MediumSecurity", "HighSecurity"];


#[derive(Debug, Serialize, Deserialize)]
pub struct GemInstall(Vec<String>);

impl GemInstall {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug)]
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

#[cfg(feature="containers")]
fn no_doc_args(ctx: &mut Context) -> Result<Vec<&'static str>, String> {
    if ctx.gem_settings.update_gem {
        Ok(vec!("--no-document"))
    } else {
        let version = gem_version(ctx)?;
        if version < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
            Ok(vec!("--no-rdoc", "--no-ri"))
        } else {
            Ok(vec!("--no-document"))
        }
    }
}

#[cfg(feature="containers")]
fn gem_version(ctx: &mut Context) -> Result<f32, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        gem_exe,
        "--version".to_owned(),
    ];

    let gem_ver = capture_command(ctx, &args, &[])
        .and_then(|out| String::from_utf8(out.stdout)
            .map_err(|e| format!("Error parsing gem version: {}", e)))
        .map_err(|e| format!("Error getting gem version: {}", e))?;

    let re = Regex::new(r#"^(\d+?\.\d+?)\."#).expect("Invalid regex");
    let version = re.captures(&gem_ver)
        .and_then(|cap| cap.get(1))
        .ok_or("Gem version was not found".to_owned())?;

    version.as_str().parse::<f32>()
        .map_err(|e| format!("Erro parsing gem version: {}", e))
}

#[cfg(feature="containers")]
fn gem_cache_dir(ctx: &mut Context) -> Result<PathBuf, String> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let args = [
        gem_exe,
        "env".to_owned(),
        "gemdir".to_owned(),
    ];

    let gem_dir = capture_command(ctx, &args, &[])
        .and_then(|out| String::from_utf8(out.stdout)
            .map_err(|e| format!("Error getting gem dir: {}", e)))?;

    Ok(Path::new(gem_dir.trim()).join("cache"))
}

fn requires_git(gemfile: &Path) -> Result<bool, String> {
    let gemfile = Path::new("/work").join(gemfile);

    let re = Regex::new(
        r#"(git .*? do)|(:(git|github|gist|bitbucket) =>)|(git_source\(.*?\))"#
    ).expect("Invalid regex");

    let gemfile_data = {
        let mut buf = String::new();
        File::open(&gemfile)
            .and_then(|mut f| f.read_to_string(&mut buf))
            .map_err(|e| format!("Error reading Gemfile ({:?}): {}", &gemfile, e))?;

        buf
    };

    Ok(re.is_match(&gemfile_data))
}

#[cfg(feature="containers")]
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
        let git_required = requires_git(&info.gemfile)?;
        if git_required {
            res.push(packages::Git);
        }
    }

    Ok(res)
}

#[cfg(feature="containers")]
pub fn install(distro: &mut Box<dyn Distribution>,
    ctx: &mut Context, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.gem_settings, None)?;
    packages::ensure_packages(distro, ctx, &features)?;

    configure(ctx)?;

    if pkgs.len() == 0 {
        return Ok(());
    }

    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = command(ctx, &gem_exe)?;
    cmd.arg("install");
    cmd.args(&["--bindir", BIN_DIR]);

    let no_doc = no_doc_args(ctx)?;
    cmd.args(&no_doc);

    cmd.args(pkgs);
    run(cmd)?;
    Ok(())
}

#[cfg(feature="containers")]
pub fn bundle(distro: &mut Box<dyn Distribution>,
    ctx: &mut Context, info: &GemBundle)
    -> Result<(), StepError>
{
    let features = scan_features(&ctx.gem_settings, Some(info))?;
    packages::ensure_packages(distro, ctx, &features)?;

    configure(ctx)?;

    let mut cmd = command(ctx, "bundle")?;
    cmd.args(&["install", "--system", "--binstubs", BIN_DIR]);

    cmd.arg("--gemfile");
    cmd.arg(&info.gemfile);

    if !info.without.is_empty() {
        cmd.arg("--without");
        cmd.args(&info.without);
    }

    if let Some(ref trust_policy) = info.trust_policy {
        if !VALID_TRUST_POLICIES.contains(&trust_policy.as_ref()) {
            return Err(From::from(format!(
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

#[cfg(feature="containers")]
pub fn configure(ctx: &mut Context) -> Result<(), String> {
    if ctx.gem_settings.gem_exe.is_none() &&
        ctx.gem_settings.update_gem
    {
        let mut args = vec!(
            DEFAULT_GEM_EXE.to_owned(),
            "update".to_owned(),
            "--system".to_owned(),
        );

        let version = gem_version(ctx)?;
        if version < GEM_VERSION_WITH_NO_DOCUMENT_OPT {
            args.extend(vec!("--no-rdoc".to_owned(), "--no-ri".to_owned()));
        } else {
            args.push("--no-document".to_owned());
        }

        // Debian based distros doesn't allow updating gem unless this flag is set
        let env = [("REALLY_GEM_UPDATE_SYSTEM", "1")];
        run_command_at_env(ctx, &args, Path::new("/work"), &env)?;
    }

    let gem_cache = gem_cache_dir(ctx)?;
    ctx.add_cache_dir(&gem_cache, "gems-cache".to_string())?;

    Ok(())
}

#[cfg(feature="containers")]
pub fn setup_bundler(ctx: &mut Context) -> Result<(), String> {
    configure(ctx)?;

    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    // It looks like recent gem update --system installs bundler
    let bundler = symlink_metadata("/vagga/root/usr/bin/bundle").is_ok();
    if !bundler {
        let mut cmd = command(ctx, gem_exe)?;
        cmd.args(&["install", "bundler"]);

        let no_doc = no_doc_args(ctx)?;
        cmd.args(&no_doc);

        run(cmd)?;
    }

    Ok(())
}

#[cfg(feature="containers")]
pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let gem_exe = ctx.gem_settings.gem_exe.clone()
        .unwrap_or(DEFAULT_GEM_EXE.to_owned());

    let mut cmd = command(ctx, gem_exe)?;
    cmd.arg("list");
    cmd.arg("--local");

    capture_output(cmd)
        .and_then(|out| {
            File::create("/vagga/container/gems-list.txt")
            .and_then(|mut f| f.write_all(&out.stdout))
            .map_err(|e| format!("Error dumping gems package list: {}", e))
        })
        .map_err(|e| warn!("Can't list gems: {}", e)).ok();
    Ok(())
}

impl BuildStep for GemInstall {
    fn name(&self) -> &'static str { "GemInstall" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("packages", &self.0);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            install(&mut guard.distro, &mut guard.ctx, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for GemConfig {
    fn name(&self) -> &'static str { "GemConfig" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("install_ruby", self.install_ruby);
        hash.opt_field("gem_exe", &self.gem_exe);
        hash.field("update_gem", self.update_gem);
        Ok(())
    }
    #[cfg(feature="containers")]
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
    fn name(&self) -> &'static str { "GemBundle" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = Path::new("/work").join(&self.gemfile);

        hash.field("gemfile", &self.gemfile);
        let gemlock = path.parent()
            .map(|dir| dir.join("Gemfile.lock"))
            .ok_or("Gemfile should be under /work")?;
        if gemlock.exists() {
            let mut lockfile = File::open(&path)
                .map_err(|e| VersionError::io(e, &gemlock))?;
            hash.file(&path, &mut lockfile)
                .map_err(|e| VersionError::io(e, &gemlock))?;
        }

        let f = File::open(&path)
            .map_err(|e| VersionError::io(e, &path))?;
        let reader = BufReader::new(f);

        for line in reader.lines() {
            let line = line
                .map_err(|e| VersionError::io(e, &path))?;
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue
            }
            hash.field("line", line);
        }

        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            bundle(&mut guard.distro, &mut guard.ctx, self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
