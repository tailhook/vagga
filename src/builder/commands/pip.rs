use std::fs::File;
use std::collections::HashSet;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::str;

use quire::validate as V;

use crate::build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
#[cfg(feature="containers")]
use crate::builder::commands::generic::{run_command_at_env, capture_command};
#[cfg(feature="containers")]
use crate::builder::context::{Context};
#[cfg(feature="containers")]
use crate::builder::distrib::Distribution;
#[cfg(feature="containers")]
use crate::builder::packages;
use crate::capsule::download;
use crate::failure::Error;
use crate::file_util::copy;
#[cfg(feature="containers")]
use crate::file_util::Dir;
#[cfg(feature="containers")]
use crate::process_util::CaptureOutput;

const PIP_HOME: &str = "/tmp/pip-install";
const PYTHON_PATH: &str = PIP_HOME;

const PIP_MIN_PYTHON_VERSION: (u8, u8, u8) = (3, 6, 0);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipConfig {
    pub find_links: Vec<String>,
    pub index_urls: Vec<String>,
    pub trusted_hosts: Vec<String>,
    pub dependencies: bool,
    pub cache_wheels: bool,
    pub install_python: bool,
    pub python_exe: Option<String>,
    pub allow_pre_releases: bool,
    pub pip_deps: Vec<String>,
}

impl PipConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("dependencies", V::Scalar::new().default(false).optional())
        .member("cache_wheels", V::Scalar::new().default(true))
        .member("find_links", V::Sequence::new(V::Scalar::new()))
        .member("index_urls", V::Sequence::new(V::Scalar::new()))
        .member("trusted_hosts", V::Sequence::new(V::Scalar::new()))
        .member("python_exe", V::Scalar::new().optional())
        .member("install_python", V::Scalar::new().default(true))
        .member("allow_pre_releases", V::Scalar::new().default(false))
        .member("pip_deps", V::Sequence::new(V::Scalar::new()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Py2Install(Vec<String>);

impl Py2Install {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Py2Requirements(PathBuf);

impl Py2Requirements {
    pub fn config() -> V::Scalar {
        V::Scalar::new().default("requirements.txt")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Py3Install(Vec<String>);

impl Py3Install {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Py3Requirements(PathBuf);

impl Py3Requirements {
    pub fn config() -> V::Scalar {
        V::Scalar::new().default("requirements.txt")
    }
}


impl Default for PipConfig {
    fn default() -> PipConfig {
        PipConfig {
            find_links: Vec::new(),
            index_urls: Vec::new(),
            trusted_hosts: Vec::new(),
            dependencies: false,
            cache_wheels: true,
            install_python: true,
            python_exe: None,
            allow_pre_releases: false,
            pip_deps: Vec::new(),
        }
    }
}


#[cfg(feature="containers")]
pub fn scan_features(settings: &PipConfig, ver: u8, pkgs: &Vec<String>)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);
    if ver == 2 {
        if settings.install_python {
            res.push(packages::Python2);
            res.push(packages::Python2Dev);
        }
        res.push(packages::PipPy2);
    } else {
        if settings.install_python {
            res.push(packages::Python3);
            res.push(packages::Python3Dev);
        }
        res.push(packages::PipPy3);
    }
    for name in pkgs.iter() {
        if name[..].starts_with("git+https") {
            res.push(packages::Git);
            res.push(packages::Https);
        } else if name[..].starts_with("git+") {
            res.push(packages::Git);
        } else if name[..].starts_with("hg+https") {
            res.push(packages::Mercurial);
            res.push(packages::Https);
        } else if name[..].starts_with("hg+") {
            res.push(packages::Mercurial);
        }
    }
    return res;
}

fn python_executable(ctx: &Context, ver: u8) -> String {
    ctx.pip_settings.python_exe.clone()
    .unwrap_or(
        (if ver == 2 {"python2"} else {"python3"})
            .to_string()
    )
}

fn python_version(ctx: &mut Context, ver: u8) -> Result<(u8, u8, u8), String> {
    let version_output = capture_command(
        ctx,
        &[
            python_executable(ctx, ver),
            "--version".to_string(),
        ],
        &[],
        if ver == 2 { CaptureOutput::Stderr } else { CaptureOutput::Stdout }
    )?;
    parse_python_version(
        &str::from_utf8(version_output.as_ref())
            .map_err(|e| format!("Error parsing python version output: {}", e))?
    )
}

fn parse_python_version(version_output: &str) -> Result<(u8, u8, u8), String> {
    let version = match version_output.rsplit_once(' ') {
        Some((_, v)) => v,
        None => version_output,
    };
    let version = match version.split_once(|c: char| !c.is_digit(10) && c != '.') {
        Some((v, _)) => v,
        None => version,
    };
    let mut version_parts = version.splitn(3, '.');
    let major = next_version_part(&mut version_parts, version_output)?;
    let minor = next_version_part(&mut version_parts, version_output)?;
    let patch = next_version_part(&mut version_parts, version_output)?;
    Ok((major, minor, patch))
}

fn next_version_part<'a>(
    version_parts: &mut impl Iterator<Item = &'a str>,
    version_output: &str,
) -> Result<u8, String> {
    version_parts.next()
        .ok_or(format!("Invalid python version: {:?}", version_output))?
        .parse()
        .map_err(|e| format!("{}: {:?}", e, version_output))
}

#[cfg(feature="containers")]
pub fn bootstrap(ctx: &mut Context, ver: u8) -> Result<(), String> {
    let mut args = vec!(
        python_executable(ctx, ver),
        "/tmp/get-pip.py".to_string(),
        format!("--target={}", PIP_HOME),
    );
    args.extend_from_slice(&ctx.pip_settings.pip_deps);
    let py_ver = python_version(ctx, ver)?;
    let _get_pip_url;
    let get_pip_url = if py_ver < PIP_MIN_PYTHON_VERSION {
        _get_pip_url = format!("https://bootstrap.pypa.io/pip/{}.{}/get-pip.py", py_ver.0, py_ver.1);
        &_get_pip_url
    } else {
        "https://bootstrap.pypa.io/get-pip.py"
    };
    let pip_inst = download::download_file(
        &mut ctx.capsule,
        &[get_pip_url],
        None,
        false
    )?;
    copy(&pip_inst, &Path::new("/vagga/root/tmp/get-pip.py"))
        .map_err(|e| format!("Error copying pip: {}", e))?;
    run_command_at_env(ctx, &args, &Path::new("/work"), &[])
}

#[cfg(feature="containers")]
fn pip_args(ctx: &mut Context, ver: u8) -> Vec<String> {
    let mut args = vec!(
        ctx.pip_settings.python_exe.clone()
        .unwrap_or((if ver == 2 { "python2" } else { "python3" }).to_string()),
        "-m".to_string(), "pip".to_string(),
        "install".to_string(),
        "--ignore-installed".to_string(),
        );
    if ctx.pip_settings.index_urls.len() > 0 {
        let mut indexes = ctx.pip_settings.index_urls.iter();
        if let Some(ref lnk) = indexes.next() {
            args.push(format!("--index-url={}", lnk));
            for lnk in indexes {
                args.push(format!("--extra-index-url={}", lnk));
            }
        }
    }
    ctx.pip_settings.trusted_hosts.iter().map(|h| {
        args.push("--trusted-host".to_string());
        args.push(h.to_string());
    }).last();
    if !ctx.pip_settings.dependencies {
        args.push("--no-deps".to_string());
    }
    for lnk in ctx.pip_settings.find_links.iter() {
        args.push(format!("--find-links={}", lnk));
    }
    if ctx.pip_settings.allow_pre_releases {
        args.push("--pre".to_string());
    }
    return args;
}

#[cfg(feature="containers")]
pub fn pip_install(distro: &mut Box<dyn Distribution>, ctx: &mut Context,
    ver: u8, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.pip_settings, ver, pkgs);
    packages::ensure_packages(distro, ctx, &features)?;
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.extend(pkgs.clone().into_iter());
    run_command_at_env(ctx, &pip_cli, &Path::new("/work"), &[
        ("PYTHONPATH", PYTHON_PATH)])
}

#[cfg(feature="containers")]
pub fn pip_requirements(distro: &mut Box<dyn Distribution>, ctx: &mut Context,
    ver: u8, reqtxt: &Path)
    -> Result<(), String>
{
    let f = File::open(&Path::new("/work").join(reqtxt))
        .map_err(|e| format!("Can't open requirements file: {}", e))?;
    let f = BufReader::new(f);
    let mut names = vec!();
    for line in f.lines() {
        let line = line
                .map_err(|e| format!("Error reading requirements: {}", e))?;
        let chunk = line[..].trim();
        // Ignore empty lines and comments
        if chunk.len() == 0 || chunk.starts_with("#") {
            continue;
        }
        names.push(chunk.to_string());
    }

    let features = scan_features(&ctx.pip_settings, ver, &names);
    packages::ensure_packages(distro, ctx, &features)?;
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.push("--requirement".to_string());
    pip_cli.push(reqtxt.to_str()
        .ok_or("Incorrect path for requirements file")?.to_string());
    run_command_at_env(ctx, &pip_cli, &Path::new("/work"), &[
        ("PYTHONPATH", PYTHON_PATH)])
}

#[cfg(feature="containers")]
pub fn configure(ctx: &mut Context) -> Result<(), String> {
    let cache_root = Path::new("/vagga/root/tmp/pip-cache");
    try_msg!(Dir::new(&cache_root).recursive(true).create(),
         "Error creating cache dir {d:?}: {err}", d=cache_root);

    ctx.add_cache_dir(Path::new("/tmp/pip-cache/http"),
                           "pip-cache-http".to_string())?;

    if ctx.pip_settings.cache_wheels {
        let cache_dir = format!("pip-cache-wheels-{}", ctx.binary_ident);
        ctx.add_cache_dir(Path::new("/tmp/pip-cache/wheels"), cache_dir)?;
    } // else just write files in tmp

    ctx.environ.insert("PIP_CACHE_DIR".to_string(),
                       "/tmp/pip-cache".to_string());
    Ok(())
}

#[cfg(feature="containers")]
pub fn freeze(ctx: &mut Context) -> Result<(), String> {
    use std::io::Write;  // TODO(tailhook) migrate whole module
    if ctx.featured_packages.contains(&packages::PipPy2) {
        let python_exe = ctx.pip_settings.python_exe.clone()
                         .unwrap_or("python2".to_string());
        capture_command(ctx, &[python_exe,
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", PYTHON_PATH)], CaptureOutput::Stdout)
            .and_then(|out| {
                File::create("/vagga/container/pip2-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            })
            .map_err(|e| warn!("Can't list pip packages: {}", e)).ok();
    }
    if ctx.featured_packages.contains(&packages::PipPy3) {
        let python_exe = ctx.pip_settings.python_exe.clone()
                         .unwrap_or("python3".to_string());
        capture_command(ctx, &[python_exe,
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", PYTHON_PATH)], CaptureOutput::Stdout)
            .and_then(|out| {
                File::create("/vagga/container/pip3-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            })
            .map_err(|e| warn!("Can't list pip packages: {}", e)).ok();
    }
    Ok(())
}

impl BuildStep for PipConfig {
    fn name(&self) -> &'static str { "PipConfig" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("find_links", &self.find_links);
        hash.field("index_urls", &self.index_urls);
        hash.field("trusted_hosts", &self.trusted_hosts);
        hash.field("dependencies", self.dependencies);
        hash.field("cache_wheels", self.cache_wheels);
        hash.field("install_python", self.install_python);
        if self.allow_pre_releases {
            hash.field("allow_pre_releases", self.allow_pre_releases);
        }
        hash.opt_field("python_exe", &self.python_exe);
        if !self.pip_deps.is_empty() {
            hash.field("pip_deps", &self.pip_deps);
        }
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        guard.ctx.pip_settings = self.clone();
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Py2Install {
    fn name(&self) -> &'static str { "Py2Install" }
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
        configure(&mut guard.ctx)?;
        if build {
            pip_install(&mut guard.distro, &mut guard.ctx, 2, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Py3Install {
    fn name(&self) -> &'static str { "Py3Install" }
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
        configure(&mut guard.ctx)?;
        if build {
            pip_install(&mut guard.distro, &mut guard.ctx, 3, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn parse_req_filename(line: &str) -> Option<&str> {
    let res = vec!["-r", "--requirement ", "--requirement=",
                   "-c", "--constraint ", "--constraint="];
    for prefix in res.iter() {
        if line.starts_with(prefix) {
            return Some(line[prefix.len()..].trim());
        }
    }
    return None;
}

fn version_req(hash: &mut Digest, fname: &Path, used: &mut HashSet<String>)
    -> Result<(), Error>
{

    let path = Path::new("/work").join(fname).canonicalize()
                   .map_err(|e| VersionError::io(e, fname))?;

    let name = format!("{:?}", path);
    if used.contains(&name[..]) {
        bail!("Cyclic requirement: {}", name);
    }

    used.insert(name);

    let f = File::open(&path).map_err(|e| VersionError::io(e, &path))?;

    let f = BufReader::new(f);
    for line in f.lines() {
        let line = line.map_err(|e| VersionError::io(e, &path))?;
        let chunk = line[..].trim();
        // Ignore empty lines and comments
        if chunk.len() == 0 || chunk.starts_with("#") {
            continue;
        }
        if let Some(req) = parse_req_filename(chunk) {
            version_req(hash, &fname.parent().unwrap().join(req), used)?;
            continue;
        }
        // Should we also ignore the order?
        hash.field("line", chunk);
    }
    Ok(())
}

impl BuildStep for Py2Requirements {
    fn name(&self) -> &'static str { "Py2Requirements" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        Ok(version_req(hash, &self.0, &mut HashSet::new())?)
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(&mut guard.ctx)?;
        if build {
            pip_requirements(&mut guard.distro, &mut guard.ctx, 2, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Py3Requirements {
    fn name(&self) -> &'static str { "Py3Requirements" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        Ok(version_req(hash, &self.0, &mut HashSet::new())?)
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(&mut guard.ctx)?;
        if build {
            pip_requirements(&mut guard.distro, &mut guard.ctx, 3, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::parse_python_version;

    #[test]
    fn test_parse_python_version() {
        assert_eq!(parse_python_version("3.9.5"), Ok((3, 9, 5)));
        assert_eq!(parse_python_version("Python 3.9.5"), Ok((3, 9, 5)));
        assert_eq!(parse_python_version("Python 3.11.0a2"), Ok((3, 11, 0)));
        assert_eq!(parse_python_version("Python 3.11.0-rc1"), Ok((3, 11, 0)));
    }
}
