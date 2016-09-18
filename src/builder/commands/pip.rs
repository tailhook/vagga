use std::fs::File;
use std::collections::HashSet;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};

use quire::validate as V;
use super::super::context::{Context};
use super::super::packages;
use super::generic::{run_command_at_env, capture_command};
use builder::distrib::Distribution;
use file_util::Dir;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(RustcDecodable, Debug, Clone)]
pub struct PipConfig {
    pub find_links: Vec<String>,
    pub index_urls: Vec<String>,
    pub trusted_hosts: Vec<String>,
    pub dependencies: bool,
    pub cache_wheels: bool,
    pub install_python: bool,
    pub python_exe: Option<String>,
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
    }
}

#[derive(Debug)]
pub struct Py2Install(Vec<String>);
tuple_struct_decode!(Py2Install);

impl Py2Install {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Debug)]
pub struct Py2Requirements(PathBuf);
tuple_struct_decode!(Py2Requirements);

impl Py2Requirements {
    pub fn config() -> V::Scalar {
        V::Scalar::new().default("requirements.txt")
    }
}

#[derive(Debug)]
pub struct Py3Install(Vec<String>);
tuple_struct_decode!(Py3Install);

impl Py3Install {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Debug)]
pub struct Py3Requirements(PathBuf);
tuple_struct_decode!(Py3Requirements);

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
        }
    }
}


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
    return args;
}

pub fn pip_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    ver: u8, pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.pip_settings, ver, pkgs);
    try!(packages::ensure_packages(distro, ctx, &features));
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.extend(pkgs.clone().into_iter());
    run_command_at_env(ctx, &pip_cli, &Path::new("/work"), &[
        ("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
}

pub fn pip_requirements(distro: &mut Box<Distribution>, ctx: &mut Context,
    ver: u8, reqtxt: &Path)
    -> Result<(), String>
{
    let f = try!(File::open(&Path::new("/work").join(reqtxt))
        .map_err(|e| format!("Can't open requirements file: {}", e)));
    let f = BufReader::new(f);
    let mut names = vec!();
    for line in f.lines() {
        let line = try!(line
                .map_err(|e| format!("Error reading requirements: {}", e)));
        let chunk = line[..].trim();
        // Ignore empty lines and comments
        if chunk.len() == 0 || chunk.starts_with("#") {
            continue;
        }
        names.push(chunk.to_string());
    }

    let features = scan_features(&ctx.pip_settings, ver, &names);
    try!(packages::ensure_packages(distro, ctx, &features));
    let mut pip_cli = pip_args(ctx, ver);
    pip_cli.push("--requirement".to_string());
    pip_cli.push(try!(reqtxt.to_str()
        .ok_or("Incorrect path for requirements file")).to_string());
    run_command_at_env(ctx, &pip_cli, &Path::new("/work"), &[
        ("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
}

pub fn configure(ctx: &mut Context) -> Result<(), String> {
    let cache_root = Path::new("/vagga/root/tmp/pip-cache");
    try_msg!(Dir::new(&cache_root).recursive(true).create(),
         "Error creating cache dir {d:?}: {err}", d=cache_root);

    try!(ctx.add_cache_dir(Path::new("/tmp/pip-cache/http"),
                           "pip-cache-http".to_string()));

    if ctx.pip_settings.cache_wheels {
        let cache_dir = format!("pip-cache-wheels-{}", ctx.binary_ident);
        try!(ctx.add_cache_dir(Path::new("/tmp/pip-cache/wheels"), cache_dir));
    } // else just write files in tmp

    ctx.environ.insert("PIP_CACHE_DIR".to_string(),
                       "/tmp/pip-cache".to_string());
    Ok(())
}

pub fn freeze(ctx: &mut Context) -> Result<(), String> {
    use std::fs::File;  // TODO(tailhook) migrate whole module
    use std::io::Write;  // TODO(tailhook) migrate whole module
    if ctx.featured_packages.contains(&packages::PipPy2) {
        let python_exe = ctx.pip_settings.python_exe.clone()
                         .unwrap_or("python2".to_string());
        try!(capture_command(ctx, &[python_exe,
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
            .and_then(|out| {
                File::create("/vagga/container/pip2-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            }));
    }
    if ctx.featured_packages.contains(&packages::PipPy3) {
        let python_exe = ctx.pip_settings.python_exe.clone()
                         .unwrap_or("python3".to_string());
        try!(capture_command(ctx, &[python_exe,
                "-m".to_string(),
                "pip".to_string(),
                "freeze".to_string(),
            ], &[("PYTHONPATH", "/tmp/non-existent:/tmp/pip-install")])
            .and_then(|out| {
                File::create("/vagga/container/pip3-freeze.txt")
                .and_then(|mut f| f.write_all(&out))
                .map_err(|e| format!("Error dumping package list: {}", e))
            }));
    }
    Ok(())
}

impl BuildStep for PipConfig {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("find_links", &self.find_links);
        hash.sequence("index_urls", &self.index_urls);
        hash.sequence("trusted_hosts", &self.trusted_hosts);
        hash.bool("dependencies", self.dependencies);
        hash.bool("cache_wheels", self.cache_wheels);
        hash.bool("install_python", self.install_python);
        hash.opt_field("python_exe", &self.python_exe);
        Ok(())
    }
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
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("Py2Install", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        try!(configure(&mut guard.ctx));
        if build {
            try!(pip_install(&mut guard.distro, &mut guard.ctx, 2, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Py3Install {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.sequence("Py3Install", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        try!(configure(&mut guard.ctx));
        if build {
            try!(pip_install(&mut guard.distro, &mut guard.ctx, 3, &self.0));
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

fn version_req(hash: &mut Digest, fname: &Path, used: &mut HashSet<String>) ->
               Result<(), VersionError> {

    let path = try!(Path::new("/work")
                       .join(fname)
                       .canonicalize()
                       .map_err(|e| VersionError::Io(e, fname.to_path_buf())));

    let name = format!("{:?}", path);
    if used.contains(&name[..]) {
        return Err(VersionError::String(
            format!("Cyclic requirement: {}", name)))
    }

    used.insert(name);

    let f = try!(File::open(&path)
                      .map_err(|e| VersionError::Io(e, path.clone())));

    let f = BufReader::new(f);
    for line in f.lines() {
        let line = try!(line.map_err(|e| VersionError::Io(e, path.clone())));
        let chunk = line[..].trim();
        // Ignore empty lines and comments
        if chunk.len() == 0 || chunk.starts_with("#") {
            continue;
        }
        if let Some(req) = parse_req_filename(chunk) {
            try!(version_req(hash,
                             &fname.parent().unwrap().join(req),
                             used));
            continue;
        }
        // Should we also ignore the order?
        hash.item(chunk);
    }
    Ok(())
}

impl BuildStep for Py2Requirements {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        version_req(hash, &self.0, &mut HashSet::new())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        try!(configure(&mut guard.ctx));
        if build {
            try!(pip_requirements(&mut guard.distro,
                &mut guard.ctx, 2, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for Py3Requirements {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        version_req(hash, &self.0, &mut HashSet::new())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        try!(configure(&mut guard.ctx));
        if build {
            try!(pip_requirements(&mut guard.distro,
                &mut guard.ctx, 3, &self.0));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
