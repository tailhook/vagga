use std::io::{BufReader, BufRead, Read};
use std::os::unix::fs::{PermissionsExt};
use std::fs::{self, File, remove_dir};
use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[cfg(feature="containers")] use libmount::BindMount;
use quire::validate as V;
use regex::Regex;
use serde_json::{Value as Json, from_reader};
use scan_dir;
#[cfg(feature="containers")] use unshare::{Stdio};

#[cfg(feature="containers")]
use builder::commands::generic::{command, run};
#[cfg(feature="containers")]
use builder::distrib::{Distribution, DistroBox};
#[cfg(feature="containers")]
use builder::commands::ubuntu;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
#[cfg(feature="containers")]
use capsule::download::download_file;
#[cfg(feature="containers")]
use container::mount::unmount;
#[cfg(feature="containers")]
use container::root::temporary_change_root;
#[cfg(feature="containers")]
use container::util::clean_dir;
#[cfg(feature="containers")]
use file_util::{safe_ensure_dir, copy, force_symlink};
#[cfg(feature="containers")]
use builder::context::{Context};
#[cfg(feature="containers")]
use builder::packages;

lazy_static! {
    static ref YARN_PATTERN: Regex = Regex::new(r#""[^"]+"|[^,]+"#).unwrap();
}


#[derive(Deserialize, Debug, Clone)]
pub struct NpmConfig {
    pub install_node: bool,
    pub install_yarn: bool,
    pub npm_exe: String,
    pub yarn_exe: String,
    pub yarn_version: Option<String>,
}

impl NpmConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("npm_exe", V::Scalar::new().default("npm"))
        .member("yarn_exe", V::Scalar::new().default("/usr/bin/yarn"))
        .member("yarn_version", V::Scalar::new().optional())
        .member("install_node", V::Scalar::new().default(true))
        .member("install_yarn", V::Scalar::new().default(true))
    }
}

#[derive(Debug, Deserialize)]
pub struct NpmInstall(Vec<String>);

impl NpmInstall {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Deserialize, Debug)]
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
            yarn_exe: "/usr/bin/yarn".to_string(),
            yarn_version: None,
        }
    }
}

#[derive(Deserialize, Debug)]
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
    let f = BufReader::new(File::open(lock_file)
        .map_err(|e| VersionError::io(e, lock_file))?);
    return _get_all_patterns(f, lock_file)
}

fn _get_all_patterns<B: BufRead>(f: B, lock_file: &Path)
    -> Result<HashSet<String>, VersionError>
{
    let mut result = HashSet::new();
    for line in f.lines() {
        let line = line.map_err(|e| VersionError::io(e, lock_file))?;
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

#[cfg(feature="containers")]
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

#[cfg(feature="containers")]
pub fn parse_feature(info: &str, features: &mut Vec<packages::Package>) {
    // Note: the info is a package name/git-url in NpmInstall but it's just
    // a version number for NpmDependencies. That's how npm works.
    if info[..].starts_with("git://") {
        features.push(packages::Git);
    } // TODO(tailhook) implement whole a lot of other npm version kinds
}

#[cfg(feature="containers")]
pub fn npm_install(distro: &mut Box<Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), StepError>
{
    ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string())?;
    let features = scan_features(&ctx.npm_settings, pkgs);
    packages::ensure_packages(distro, ctx, &features)?;
    if !ctx.npm_configured {
        if let Some(ubuntu) = distro.downcast_ref::<ubuntu::Distro>() {
            match ubuntu.codename.as_ref().map(|x| &x[..]) {
                | Some("trusty")
                | Some("precise")
                => {
                    // Old npm requires switching
                    // to system certificates manually
                    let mut cmd = command(ctx, &ctx.npm_settings.npm_exe)?;
                    cmd.arg("config");
                    cmd.arg("set");
                    cmd.arg("ca");
                    cmd.arg("");
                    run(cmd)?;
                }
                | Some(_)
                | None
                => {}
            }
        }
    }

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

#[cfg(feature="containers")]
fn scan_dic(json: &Json, key: &str,
    packages: &mut Vec<String>, features: &mut Vec<packages::Package>)
    -> Result<(), StepError>
{
    match json.get(key) {
        Some(&Json::Object(ref ob)) => {
            for (k, v) in ob {
                if !v.is_string() {
                    return Err(StepError::Compat(format!(
                        "Package {:?} has wrong version {:?}", k, v)));
                }
                let s = v.as_str().unwrap();
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

#[cfg(feature="containers")]
pub fn npm_deps(distro: &mut Box<Distribution>, ctx: &mut Context,
    info: &NpmDependencies)
    -> Result<(), StepError>
{
    ctx.add_cache_dir(Path::new("/tmp/npm-cache"),
                           "npm-cache".to_string())?;
    let mut features = scan_features(&ctx.npm_settings, &Vec::new());

    let json = File::open(&Path::new("/work").join(&info.file))
        .map_err(|e| format!("Error opening file {:?}: {}", info.file, e))
        .and_then(|mut f| from_reader(&mut f)
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

#[cfg(feature="containers")]
pub fn list(ctx: &mut Context) -> Result<(), StepError> {
    let path = Path::new("/vagga/container/npm-list.txt");
    let file = File::create(&path)
        .map_err(|e| StepError::Write(path.to_path_buf(), e))?;
    let mut cmd = command(ctx, &ctx.npm_settings.npm_exe)?;
    cmd.arg("ls");
    cmd.arg("--global");
    cmd.stdout(Stdio::from_file(file));
    run(cmd)
        .map_err(|e| warn!("Can't list npm packages: {}", e)).ok();
    Ok(())
}

fn npm_hash_deps(data: &Json, key: &str, hash: &mut Digest) {
    let deps = data.get(key);
    if let Some(&Json::Object(ref ob)) = deps {
        // Note the BTree is sorted on its own
        for (key, val) in ob {
            hash.field(key, val.as_str().unwrap_or("*"));
        }
    }
}

impl BuildStep for NpmConfig {
    fn name(&self) -> &'static str { "NpmConfig" }
    #[cfg(feature="containers")]
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
        hash.opt_field("yarn_version", &self.yarn_version);
        Ok(())
    }
    #[cfg(feature="containers")]
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
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let path = Path::new("/work").join(&self.file);
        File::open(&path)
            .map_err(|e| VersionError::io(e, &path))
        .and_then(|mut f| from_reader(&mut f)
            .map_err(|e| format_err!("bad json in {:?}: {}", &path, e).into()))
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
    #[cfg(feature="containers")]
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

#[cfg(feature="containers")]
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

#[cfg(feature="containers")]
pub fn setup_yarn(ctx: &mut Context)
    -> Result<(), String>
{
    let ver = match ctx.npm_settings.yarn_version {
        Some(ref ver) => ver.trim_left_matches('v').to_string(),
        None => {
            let path = download_file(&mut ctx.capsule,
                &["https://yarnpkg.com/latest-version"],
                None, true)?;
            let mut ver = String::with_capacity(10);
            // TODO(tailhook) remove cached latest-version
            //                but have to do it race-less
            File::open(&path)
                .and_then(|mut f| f.read_to_string(&mut ver))
                .map_err(|e| format!(
                    "error reading yarn's latest version: {}", e))?;
            ver
        }
    };
    let ver = ver.trim();
    let link = format!("https://github.com/yarnpkg/yarn/\
                       releases/download/v{0}/yarn-{0}.js", ver);
    let filename = download_file(&mut ctx.capsule, &[&link], None, false)?;
    copy(&filename, "/vagga/root/usr/bin/yarn")
        .map_err(|e| format!("Error copying {:?} to {:?}: {}",
            &filename, "/vagga/root/usr/bin/yarn", e))?;
    fs::set_permissions("/vagga/root/usr/bin/yarn",
        fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("Error setting permissions of {:?}: {}",
            "/vagga/root/usr/bin/yarn", e))?;
    Ok(())
}

fn check_deps(deps: Option<&Json>, patterns: &HashSet<String>) -> bool {
    let items = match deps.and_then(|x| x.as_object()) {
        Some(items) => items,
        None => return true,
    };
    for (key, value) in items.iter() {
        let val = match value.as_str() {
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
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("production", self.production);
        let lock_file = Path::new("/work").join(&self.dir).join("yarn.lock");
        let package = Path::new("/work").join(&self.dir).join("package.json");
        if lock_file.exists() {
            let data: Json = from_reader(
                &mut File::open(&package)
                    .map_err(|e| VersionError::io(e, &package))?)
                .map_err(|e| format_err!("bad json in {:?}: {}", package, e))?;
            let patterns = get_all_patterns(&lock_file)?;

            // This is what yarn as of v0.23.0, i.e. checks whether all
            // dependencies are in lockfile
            if !check_deps(data.get("dependencies"), &patterns) {
                return Err(VersionError::New);
            }
            npm_hash_deps(&data, "dependencies", hash);
            if !self.production {
                if !check_deps(data.get("devDependencies"), &patterns) {
                    return Err(VersionError::New);
                }
                npm_hash_deps(&data, "devDependencies", hash);
            }
            if self.optional {
                if !check_deps(data.get("optionalDependencies"), &patterns) {
                    return Err(VersionError::New);
                }
                npm_hash_deps(&data, "optionalDependencies", hash);
            }

            let mut file = File::open(&lock_file)
                .map_err(|e| VersionError::io(e, &lock_file))?;
            hash.file(&lock_file, &mut file)
                .map_err(|e| VersionError::io(e, &lock_file))?;
            Ok(())
        } else {
            debug!("No lockfile exits at {:?}", lock_file);
            Err(VersionError::New)
        }
    }
    #[cfg(feature="containers")]
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

            let bad_modules = Path::new("/vagga/root/work")
                .join(&self.dir)
                .join("node_modules");
            let modules_exist = bad_modules.is_dir();
            if !modules_exist {
                safe_ensure_dir(&bad_modules)?;
            }
            safe_ensure_dir(Path::new("/tmp"))?;
            safe_ensure_dir(Path::new("/tmp/yarn-modules"))?;

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

            BindMount::new("/tmp/yarn-modules", &bad_modules).mount()?;

            let r1 = run(cmd);

            let r2 = temporary_change_root::<_, _, _, String>("/vagga/root",
                || {
                    let dir = Path::new("/work").join(&self.dir)
                        .join("node_modules/.bin");
                    if dir.exists() {
                        scan_dir::ScanDir::files().read(dir, |iter| {
                            for (entry, name) in iter {
                                let res = fs::read_link(entry.path())
                                    .map_err(|e| format!(
                                        "Readlink error: {}", e))?;
                                force_symlink(
                                        &res,
                                        &Path::new("/usr/bin").join(&name))
                                    .map_err(|e| format!(
                                        "Error symlinking: {}", e))?;
                            }
                            Ok(())
                        })
                        .map_err(|e| format!("Can't scan bin dir: {}", e))
                        .and_then(|v| v)?;
                    }
                    Ok(())
                });

            unmount(&bad_modules)?;
            if !modules_exist {
                remove_dir(&bad_modules)
                    .map_err(|e| format!("Can't remove node_modules: {}", e))?;
            }
            clean_dir("/tmp/yarn-modules", true)?;

            r1?;
            r2?;
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
