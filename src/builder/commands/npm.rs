use std::io::{BufReader, BufRead};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::os::unix::fs::{symlink};
use std::collections::HashSet;

use libmount::BindMount;
use quick_error::ResultExt;
use quire::validate as V;
use regex::Regex;
use rustc_serialize::json::Json;
use unshare::{Stdio};

use builder::commands::generic::{command, run};
use builder::commands::tarcmd::unpack_subdir;
use builder::distrib::{Distribution, DistroBox};
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use capsule::download::download_file;
use container::mount::unmount;
use file_util::safe_ensure_dir;
use super::super::context::{Context};
use super::super::packages;

lazy_static! {
    static ref YARN_PATTERN: Regex = Regex::new(r#""[^"]+"|[^,]+"#).unwrap();
}


#[derive(RustcDecodable, Debug, Clone)]
pub struct NpmConfig {
    pub install_node: bool,
    pub install_yarn: bool,
    pub npm_exe: String,
    pub yarn_exe: String,
}

impl NpmConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("npm_exe", V::Scalar::new().default("npm"))
        .member("yarn_exe", V::Scalar::new().default("/usr/lib/yarn/bin/yarn"))
        .member("install_node", V::Scalar::new().default(true))
        .member("install_yarn", V::Scalar::new().default(true))
    }
}

#[derive(Debug)]
pub struct NpmInstall(Vec<String>);
tuple_struct_decode!(NpmInstall);

impl NpmInstall {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(RustcDecodable, Debug)]
pub struct NpmDependencies {
    pub file: PathBuf,
    pub package: bool,
    pub dev: bool,
    pub peer: bool,
    pub bundled: bool,
    pub optional: bool,
}

impl NpmDependencies {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("file", V::Scalar::new().default("package.json"))
        .member("package", V::Scalar::new().default(true))
        .member("dev", V::Scalar::new().default(true))
        .member("peer", V::Scalar::new().default(false))
        .member("bundled", V::Scalar::new().default(true))
        .member("optional", V::Scalar::new().default(false))
    }
}

impl Default for NpmConfig {
    fn default() -> NpmConfig {
        NpmConfig {
            install_node: true,
            install_yarn: true,
            npm_exe: "npm".to_string(),
            yarn_exe: "/usr/lib/yarn/bin/yarn".to_string(),
        }
    }
}

#[derive(RustcDecodable, Debug)]
pub struct YarnDependencies {
    pub dir: PathBuf,
    pub production: bool,
    pub optional: bool,
}

impl YarnDependencies {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("dir", V::Directory::new().absolute(false).default("."))
        .member("production", V::Scalar::new().default(false))
        .member("optional", V::Scalar::new().default(false))
    }
}

fn get_all_patterns(lock_file: &Path)
    -> Result<HashSet<String>, VersionError>
{
    let f = BufReader::new(File::open(lock_file).context(lock_file)?);
    return _get_all_patterns(f, lock_file)
}

fn _get_all_patterns<B: BufRead>(f: B, lock_file: &Path)
    -> Result<HashSet<String>, VersionError>
{
    let mut result = HashSet::new();
    for line in f.lines() {
        let line = line.context(lock_file)?;
        if line.starts_with(" ") || line.starts_with("#") {
            continue;
        }
        if !line.ends_with(":") {
            continue;
        }
        for item in YARN_PATTERN.find_iter(&line[..line.len()-1]) {
            let mut item = item.as_str().trim();
            if item.starts_with('"') && item.ends_with('"') {
                item = &item[1..item.len()-1];
            }
            result.insert(item.to_string());
        }
    }
    Ok(result)
}

fn scan_features(settings: &NpmConfig, pkgs: &Vec<String>)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);
    if settings.install_node {
        res.push(packages::NodeJs);
        res.push(packages::NodeJsDev);
        res.push(packages::Npm);
    }
    for name in pkgs.iter() {
        parse_feature(&name, &mut res);
    }
    return res;
}

pub fn parse_feature(info: &str, features: &mut Vec<packages::Package>) {
    // Note: the info is a package name/git-url in NpmInstall but it's just
    // a version number for NpmDependencies. That's how npm works.
    if info[..].starts_with("git://") {
        features.push(packages::Git);
    } // TODO(tailhook) implement whole a lot of other npm version kinds
}

pub fn npm_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), StepError>
{
    ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string())?;
    let features = scan_features(&ctx.npm_settings, pkgs);
    packages::ensure_packages(distro, ctx, &features)?;

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = command(ctx, &ctx.npm_settings.npm_exe)?;
    cmd.arg("install");
    cmd.arg("--global");
    cmd.arg("--user=root");
    cmd.arg("--cache=/tmp/npm-cache");
    cmd.args(pkgs);
    run(cmd)
}

fn scan_dic(json: &Json, key: &str,
    packages: &mut Vec<String>, features: &mut Vec<packages::Package>)
    -> Result<(), StepError>
{
    match json.find(key) {
        Some(&Json::Object(ref ob)) => {
            for (k, v) in ob {
                if !v.is_string() {
                    return Err(StepError::Compat(format!(
                        "Package {:?} has wrong version {:?}", k, v)));
                }
                let s = v.as_string().unwrap();
                parse_feature(&s, features);
                packages.push(format!("{}@{}", k, s));
                // TODO(tailhook) check the feature
            }
            Ok(())
        }
        None => {
            Ok(())
        }
        Some(_) => {
            Err(StepError::Compat(format!(
                "The {:?} is not a mapping (JSON object)", key)))
        }
    }
}

pub fn npm_deps(distro: &mut Box<Distribution>, ctx: &mut Context,
    info: &NpmDependencies)
    -> Result<(), StepError>
{
    ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string())?;
    let mut features = scan_features(&ctx.npm_settings, &Vec::new());

    let json = File::open(&Path::new("/work").join(&info.file))
        .map_err(|e| format!("Error opening file {:?}: {}", info.file, e))
        .and_then(|mut f| Json::from_reader(&mut f)
        .map_err(|e| format!("Error parsing json {:?}: {}", info.file, e)))?;
    let mut packages = vec![];

    if info.package {
        scan_dic(&json, "dependencies", &mut packages, &mut features)?;
    }
    if info.dev {
        scan_dic(&json, "devDependencies", &mut packages, &mut features)?;
    }
    if info.peer {
        scan_dic(&json, "peerDependencies",
            &mut packages, &mut features)?;
    }
    if info.bundled {
        scan_dic(&json, "bundledDependencies",
            &mut packages, &mut features)?;
        scan_dic(&json, "bundleDependencies",
            &mut packages, &mut features)?;
    }
    if info.optional {
        scan_dic(&json, "optionalDependencies",
            &mut packages, &mut features)?;
    }

    packages::ensure_packages(distro, ctx, &features)?;

    if packages.len() == 0 {
        return Ok(());
    }

    let mut cmd = command(ctx, &ctx.npm_settings.npm_exe)?;
    cmd.arg("install");
    cmd.arg("--global");
    cmd.arg("--user=root");
    cmd.arg("--cache=/tmp/npm-cache");
    cmd.args(&packages);
    run(cmd)
}

pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let path = Path::new("/vagga/container/npm-list.txt");
    let file = File::create(&path)
        .map_err(|e| StepError::Write(path.to_path_buf(), e))?;
    let mut cmd = command(ctx, &ctx.npm_settings.npm_exe)?;
    cmd.arg("ls");
    cmd.arg("--global");
    cmd.stdout(Stdio::from_file(file));
    run(cmd)
}

fn npm_hash_deps(data: &Json, key: &str, hash: &mut Digest) {
    let deps = data.find(key);
    if let Some(&Json::Object(ref ob)) = deps {
        // Note the BTree is sorted on its own
        for (key, val) in ob {
            hash.field(key, val.as_string().unwrap_or("*"));
        }
    }
}

impl BuildStep for NpmConfig {
    fn name(&self) -> &'static str { "NpmConfig" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("npm_exe", &self.npm_exe);
        hash.field("install_node", self.install_node);
        if !self.install_yarn {
            hash.field("install_yarn", self.install_yarn);
        }
        if self.yarn_exe != "yarn" {
            hash.field("yarn_exe", &self.yarn_exe);
        }
        Ok(())
    }
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        guard.ctx.npm_settings = self.clone();
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for NpmInstall {
    fn name(&self) -> &'static str { "NpmInstall" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("packages", &self.0);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        guard.distro.npm_configure(&mut guard.ctx)?;
        if build {
            npm_install(&mut guard.distro, &mut guard.ctx, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for NpmDependencies {
    fn name(&self) -> &'static str { "NpmDependencies" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = Path::new("/work").join(&self.file);
        File::open(&path).map_err(|e| VersionError::Io(e, path.clone()))
        .and_then(|mut f| Json::from_reader(&mut f)
            .map_err(|e| VersionError::Json(e, path.to_path_buf())))
        .map(|data| {
            if self.package {
                npm_hash_deps(&data, "dependencies", hash);
            }
            if self.dev {
                npm_hash_deps(&data, "devDependencies", hash);
            }
            if self.peer {
                npm_hash_deps(&data, "peerDependencies", hash);
            }
            if self.bundled {
                npm_hash_deps(&data, "bundledDependencies", hash);
                npm_hash_deps(&data, "bundleDependencies", hash);
            }
            if self.optional {
                npm_hash_deps(&data, "optionalDependencies", hash);
            }
        })
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        guard.distro.npm_configure(&mut guard.ctx)?;
        if build {
            npm_deps(&mut guard.distro, &mut guard.ctx, &self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn yarn_scan_features(settings: &NpmConfig, pkgs: &Vec<String>)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::BuildEssential);
    if settings.install_node {
        res.push(packages::NodeJs);
        res.push(packages::NodeJsDev);
    }
    res.push(packages::Yarn);
    for name in pkgs.iter() {
        parse_feature(&name, &mut res);
    }
    return res;
}

pub fn setup_yarn(ctx: &mut Context)
    -> Result<(), String>
{
    let filename = download_file(&mut ctx.capsule,
        &["https://yarnpkg.com/latest.tar.gz"], None)?;
    unpack_subdir(ctx, &filename,
        &Path::new("/vagga/root/usr/lib/yarn"), Path::new("dist"))?;
    symlink("/usr/lib/yarn/bin/yarn", "/vagga/root/usr/bin/yarn")
        .map_err(|e| format!("Can't create yarn symlink: {}", e))?;
    Ok(())
}

fn check_deps(deps: Option<&Json>, patterns: &HashSet<String>) -> bool {
    let items = match deps.and_then(|x| x.as_object()) {
        Some(items) => items,
        None => return true,
    };
    for (key, value) in items.iter() {
        let val = match value.as_string() {
            Some(x) => x,
            None => continue,
        };
        let item = format!("{}@{}", key, val);
        if !patterns.contains(&item) {
            debug!("Yarn deps: no pattern {:?} in lockfile", item);
            return false;
        }
    }
    return true;
}

impl BuildStep for YarnDependencies {
    fn name(&self) -> &'static str { "YarnDependencies" }
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("production", self.production);
        let lock_file = Path::new("/work").join(&self.dir).join("yarn.lock");
        let package = Path::new("/work").join(&self.dir).join("package.json");
        if lock_file.exists() {
            let data = Json::from_reader(
                &mut File::open(&package).context(&package)?)
                .context(&package)?;
            let patterns = get_all_patterns(&lock_file)?;

            // This is what yarn as of v0.23.0, i.e. checks whether all
            // dependencies are in lockfile
            if !check_deps(data.find("dependencies"), &patterns) {
                return Err(VersionError::New);
            }
            npm_hash_deps(&data, "dependencies", hash);
            if !self.production {
                if !check_deps(data.find("devDependencies"), &patterns) {
                    return Err(VersionError::New);
                }
                npm_hash_deps(&data, "devDependencies", hash);
            }
            if self.optional {
                if !check_deps(data.find("optionalDependencies"), &patterns) {
                    return Err(VersionError::New);
                }
                npm_hash_deps(&data, "optionalDependencies", hash);
            }

            let mut file = File::open(&lock_file).context(&lock_file)?;
            hash.file(&lock_file, &mut file).context(&lock_file)?;
            Ok(())
        } else {
            debug!("No lockfile exits at {:?}", lock_file);
            Err(VersionError::New)
        }
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let base_dir = Path::new("/work").join(&self.dir);

            guard.ctx.add_cache_dir(Path::new("/tmp/yarn-cache"),
                                    "yarn-cache".to_string())?;
            let features = yarn_scan_features(
                &guard.ctx.npm_settings, &Vec::new());
            packages::ensure_packages(
                &mut guard.distro, &mut guard.ctx, &features)?;

            // We need to hide `node_modules/.yarn-integrity` so that yarn
            // skip this directory
            // At least this is how it works in yarn v0.23.0
            let bad_modules = Path::new("/vagga/root/work")
                .join(&self.dir)
                .join("node_modules");
            let modules_mount = if bad_modules.is_dir() {
                safe_ensure_dir(Path::new("/vagga/empty"))?;
                BindMount::new("/vagga/empty", &bad_modules).mount()?;
                Some(bad_modules)
            } else {
                None
            };

            let mut cmd = command(&guard.ctx,
                &guard.ctx.npm_settings.yarn_exe)?;
            cmd.current_dir(&base_dir);
            cmd.arg("install");
            // We use --no-progress because build process is run without
            // controlling terminal, and yarn can't determine its width to
            // display progressbar corectly
            cmd.arg("--no-progress");
            cmd.arg("--modules-folder=/usr/lib/node_modules");
            cmd.arg("--cache-folder=/tmp/yarn-cache");
            // TODO(tailhook) figure out how to pass frozen-lockfile if needed
            // cmd.arg("--frozen-lockfile");
            if self.production {
                cmd.arg("--production");
            }
            let result = run(cmd);

            if let Some(bad_modules) = modules_mount {
                unmount(&bad_modules)?;
            }

            result?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::_get_all_patterns;
    use std::io::Cursor;
    use std::path::Path;
    use std::collections::HashSet;

    fn parse(text: &str) -> HashSet<String> {
        _get_all_patterns(Cursor::new(text), &Path::new("<string>")).unwrap()
    }

    #[test]
    fn yarn_simple() {
        assert_eq!(
            parse("babel-eslint@^6.1.2:\n"),
            vec![
                "babel-eslint@^6.1.2"
            ].iter().map(ToString::to_string).collect());
    }
    #[test]
    fn yarn_pair() {
        assert_eq!(
            parse("babel-core@^6.24.1, babel-core@^6.25.0:\n"),
            vec![
                "babel-core@^6.24.1",
                "babel-core@^6.25.0",
            ].iter().map(ToString::to_string).collect());
    }
    #[test]
    fn yarn_quoted() {
        assert_eq!(
            parse("\"@types/node@^6.0.46\":\n"),
            vec![
                "@types/node@^6.0.46",
            ].iter().map(ToString::to_string).collect());
    }
}
