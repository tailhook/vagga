use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix;

#[cfg(feature="containers")] use unshare::{Command};
use scan_dir::{self, ScanDir};
use regex::Regex;
use serde_json::{Value as Json, from_reader};
use quire::validate as V;

#[cfg(feature="containers")] use builder::context::{Context};
#[cfg(feature="containers")] use builder::packages;
#[cfg(feature="containers")]
use builder::commands::generic::{run_command, capture_command};
#[cfg(feature="containers")] use builder::distrib::Distribution;
#[cfg(feature="containers")] use builder::commands::generic::{command, run};
#[cfg(feature="containers")] use capsule::download;
#[cfg(feature="containers")] use file_util::Dir;
#[cfg(feature="containers")] use process_util::capture_output;
#[cfg(feature="containers")] use file_util;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};

const DEFAULT_RUNTIME: &'static str = "/usr/bin/php";
const DEFAULT_INCLUDE_PATH: &'static str = ".:/usr/local/lib/composer";

const COMPOSER_HOME: &'static str = "/usr/local/lib/composer";
const COMPOSER_CACHE: &'static str = "/tmp/composer-cache";
const COMPOSER_VENDOR_DIR: &'static str = "/usr/local/lib/composer/vendor";
const COMPOSER_BIN_DIR: &'static str = "/usr/local/bin";
const COMPOSER_BOOTSTRAP: &'static str = "https://getcomposer.org/installer";
const COMPOSER_SELF_CACHE: &'static str = "/tmp/composer-self-cache";

const LOCKFILE_RELEVANT_KEYS: &'static [&'static str] = &[
    "name",
    "version",
    "source",
    "dist",
    "extra",
    "autoload",
];

const CONF_D: &'static str = "conf.d";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComposerConfig {
    // It is used 'runtime' instead of 'php' in order to support hhvm in the future
    pub install_runtime: bool,
    pub install_dev: bool,
    pub runtime_exe: Option<String>,
    pub include_path: Option<String>,
    pub keep_composer: bool,
    pub vendor_dir: Option<PathBuf>,
}

impl ComposerConfig {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("install_runtime", V::Scalar::new().default(true))
        .member("install_dev", V::Scalar::new().default(false))
        .member("runtime_exe", V::Scalar::new().optional())
        .member("include_path", V::Scalar::new().optional())
        .member("keep_composer", V::Scalar::new().default(false))
        .member("vendor_dir", V::Directory::new().optional())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComposerInstall(Vec<String>);

impl ComposerInstall {
    pub fn config() -> V::Sequence<'static> {
        V::Sequence::new(V::Scalar::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComposerDependencies {
    pub working_dir: Option<String>,
    pub dev: bool,
    pub prefer: Option<String>,
    pub ignore_platform_reqs: bool,
    pub no_autoloader: bool,
    pub no_scripts: bool,
    pub no_plugins: bool,
    pub optimize_autoloader: bool,
    pub classmap_authoritative: bool,
}

impl ComposerDependencies {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("working_dir", V::Scalar::new().optional())
        .member("dev", V::Scalar::new().default(true))
        .member("prefer", V::Scalar::new().optional())
        .member("ignore_platform_reqs", V::Scalar::new().default(false))
        .member("no_autoloader", V::Scalar::new().default(false))
        .member("no_scripts", V::Scalar::new().default(false))
        .member("no_plugins", V::Scalar::new().default(false))
        .member("optimize_autoloader", V::Scalar::new().default(false))
        .member("classmap_authoritative", V::Scalar::new().default(false))
    }
}

impl Default for ComposerConfig {
    fn default() -> Self {
        ComposerConfig {
            install_runtime: true,
            install_dev: false,
            runtime_exe: None,
            include_path: None,
            keep_composer: false,
            vendor_dir: None,
        }
    }
}

#[cfg(feature="containers")]
fn scan_features(settings: &ComposerConfig)
    -> Vec<packages::Package>
{
    let mut res = vec!();
    res.push(packages::Https);
    res.push(packages::Composer);
    if settings.install_runtime {
        res.push(packages::Php);
        if settings.install_dev {
            res.push(packages::BuildEssential);
            res.push(packages::PhpDev)
        }
    }
    // Probably it's not it worth trying to figure out whether we need Git or Mercurial and it is
    // more likely that a php project is using Git, therefore it is reasonable to simply assume we
    // always need Git
    res.push(packages::Git);
    return res;
}

#[cfg(feature="containers")]
fn composer_cmd(ctx: &mut Context) -> Result<Command, StepError> {
    let runtime = ctx.composer_settings
        .runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());
    let mut cmd = command(ctx, runtime)?;
    cmd.arg("/usr/local/bin/composer");
    cmd.arg("--no-interaction");
    Ok(cmd)
}

#[cfg(feature="containers")]
pub fn composer_install(distro: &mut Box<dyn Distribution>, ctx: &mut Context,
    pkgs: &Vec<String>)
    -> Result<(), String>
{
    let features = scan_features(&ctx.composer_settings);
    packages::ensure_packages(distro, ctx, &features)?;

    if pkgs.len() == 0 {
        return Ok(());
    }

    let mut cmd = composer_cmd(ctx)?;
    cmd.args(&["global", "require", "--prefer-dist", "--update-no-dev"]);
    cmd.args(pkgs);
    run(cmd)?;
    Ok(())
}

#[cfg(feature="containers")]
pub fn composer_dependencies(distro: &mut Box<dyn Distribution>,
    ctx: &mut Context, info: &ComposerDependencies)
    -> Result<(), StepError>
{
    let features = scan_features(&ctx.composer_settings);
    packages::ensure_packages(distro, ctx, &features)?;

    let mut cmd = composer_cmd(ctx)?;
    cmd.arg("install");
    if let Some(ref dir) = info.working_dir {
        cmd.arg(format!("--working-dir={}", dir));
    }
    if !info.dev { cmd.arg("--no-dev"); }
    if info.ignore_platform_reqs { cmd.arg("--ignore-platform-reqs"); }
    if info.no_autoloader { cmd.arg("--no_autoloader"); }
    if info.no_scripts { cmd.arg("--no-scripts"); }
    if info.no_plugins { cmd.arg("--no-plugins"); }
    if info.optimize_autoloader { cmd.arg("--optimize-autoloader"); }

    match info.prefer {
        Some(ref p) if p == "dist" => { cmd.arg("--prefer-dist"); },
        Some(ref p) if p == "source" => { cmd.arg("--prefer-source"); },
        Some(ref p) => return Err(From::from(format!(
            "Value of 'ComposerDependencies.prefer' must be either \
            'source' or 'dist', '{}' given", p
        ))),
        _ => {}
    }

    run(cmd)
}

#[cfg(feature="containers")]
pub fn configure(ctx: &mut Context) -> Result<(), String> {
    ctx.add_cache_dir(Path::new(COMPOSER_CACHE),
                           "composer-cache".to_string())?;

    ctx.add_cache_dir(Path::new(COMPOSER_SELF_CACHE),
                           "composer-self-cache".to_owned())?;

    let vendor_dir = ctx.composer_settings.vendor_dir.as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| COMPOSER_VENDOR_DIR.to_owned());

    ctx.environ.insert("COMPOSER_VENDOR_DIR".to_owned(), vendor_dir);

    ctx.environ.insert("COMPOSER_HOME".to_owned(),
                       COMPOSER_HOME.to_owned());
    ctx.environ.insert("COMPOSER_BIN_DIR".to_owned(),
                       COMPOSER_BIN_DIR.to_owned());
    ctx.environ.insert("COMPOSER_CACHE_DIR".to_owned(),
                       COMPOSER_CACHE.to_owned());
    ctx.environ.insert("COMPOSER_ALLOW_SUPERUSER".to_owned(),
                       "1".to_owned());

    Ok(())
}

#[cfg(feature="containers")]
pub fn bootstrap(ctx: &mut Context) -> Result<(), String> {
    try_msg!(Dir::new(COMPOSER_HOME).recursive(true).create(),
        "Error creating composer home dir {d:?}: {err}", d=COMPOSER_HOME);

    let runtime_exe = ctx.composer_settings.runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());

    let default_runtime = Path::new("/vagga/root")
        .join(DEFAULT_RUNTIME.trim_start_matches('/'));
    // if using a custom runtime, link it to '/usr/bin/php' so that packages expecting the `php`
    // binary to be on PATH can work correctly
    if runtime_exe != DEFAULT_RUNTIME {
        unix::fs::symlink(&runtime_exe, &default_runtime)
            .or_else(|e| { // Ignore error if link destination already exists
                if e.kind() != io::ErrorKind::AlreadyExists { Err(e) }
                else { Ok(()) }
            })
            .map_err(|e| format!("Error creating symlink '{s}' to '{d}': {err}",
                                 s=runtime_exe, d=DEFAULT_RUNTIME, err=e))?;
    }

    let cached_composer = format!("/vagga/root{}/composer.phar", COMPOSER_SELF_CACHE);
    if Path::new(&cached_composer).exists() {
        update_composer(ctx, &runtime_exe)?;
    } else {
        install_composer(ctx, &runtime_exe)?;
    }

    file_util::copy(cached_composer, "/vagga/root/usr/local/bin/composer")
        .map_err(|e| format!("Error copying composer binary: {}", e))?;

    if ctx.composer_settings.install_runtime {
        setup_include_path(ctx)?;
    }

    Ok(())
}

#[cfg(feature="containers")]
fn update_composer(ctx: &mut Context, runtime: &str) -> Result<(), String> {
    let args = [
        runtime.to_owned(),
        format!("{}/composer.phar", COMPOSER_SELF_CACHE),
        "self-update".to_owned(),
        "--clean-backups".to_owned(),
    ];

    run_command(ctx, &args)
}

#[cfg(feature="containers")]
fn install_composer(ctx: &mut Context, runtime: &str) -> Result<(), String> {
    let composer_inst = download::download_file(&mut ctx.capsule,
        &[COMPOSER_BOOTSTRAP], None, false)?;
    file_util::copy(&composer_inst,
                         &Path::new("/vagga/root/tmp/composer-setup.php"))
        .map_err(|e| format!("Error copying composer installer: {}", e))?;

    let args = [
        runtime.to_owned(),
        "/tmp/composer-setup.php".to_owned(),
        format!("--install-dir={}", COMPOSER_SELF_CACHE),
    ];

    run_command(ctx, &args)
}

#[cfg(feature="containers")]
fn setup_include_path(ctx: &mut Context) -> Result<(), String> {
    let vagga_ini_content = {
        let include_path = ctx.composer_settings
            .include_path.clone()
            .unwrap_or(DEFAULT_INCLUDE_PATH.to_owned());
        format!("include_path={}", include_path)
    };

    let conf_dirs = try_msg!(find_conf_dirs(),
                             "Error listing PHP configuration directories: {err}");

    for conf_d in conf_dirs.iter() {
        // create vagga.ini file
        create_vagga_ini(&conf_d.join("vagga.ini"), &vagga_ini_content)?;
    }

    if !conf_dirs.is_empty() {
        // vagga.ini file(s) created, we're done here
        return Ok(())
    }

    // If we didn't find any conf.d, ask 'php --ini'
    let conf_d = ask_php_for_conf_d(ctx)?;

    // create conf.d
    if !conf_d.exists() {
        Dir::new(&conf_d).recursive(true).create()
        .map_err(|e| format!("Error creating directory {:?}: {}", conf_d, e))?;
    }

    // create vagga.ini file
    create_vagga_ini(&conf_d.join("vagga.ini"), &vagga_ini_content)?;

    Ok(())
}

#[cfg(feature="containers")]
fn create_vagga_ini(location: &Path, content: &str) -> Result<(), String> {
    File::create(location)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| format!("Error creating file {:?}: {}", location, e))
}

#[cfg(feature="containers")]
fn find_conf_dirs() -> Result<Vec<PathBuf>, scan_dir::Error> {
    // find php main config directory (/etc/php or /etc/php5 or both)
    let etc_php: Vec<PathBuf> =
        ScanDir::dirs().skip_symlinks(true).read("/vagga/root/etc", |iter| {
            iter.filter(|&(_, ref name)| name.starts_with("php"))
            .map(|(ref entry, _)| entry.path())
            .collect()
        })
    ?;

    // get subdirectories of main php config directory
    let mut etc_php_dirs = Vec::new();
    for path in etc_php.iter() {
        ScanDir::dirs().skip_symlinks(true).read(path, |iter| {
            for (ref entry, _) in iter {
                etc_php_dirs.push(entry.path())
            }
        })?;
    }

    // In ubuntu xenial, /etc/php directory structure was changed, now it's like:
    // /etc/php
    // └── 7.0
    //     ├── cli
    //     ├── fpm
    //     └── mods-available
    // instead of:
    // /etc/php5
    // ├── cli
    // ├── fpm
    // └── mods-available
    // because of the extra directory for the php version, we need to search one more
    // level down the directory tree, otherwise the `conf.d` directory would not be
    // found in ubuntu xenial
    let mut etc_php_subdirs = Vec::new();
    for path in etc_php_dirs.iter() {
        ScanDir::dirs().skip_symlinks(true).read(path, |iter| {
            for (ref entry, _) in iter {
                let path = entry.path();
                if path.ends_with(CONF_D) {
                    etc_php_subdirs.push(entry.path());
                } else if path.join(CONF_D).exists() {
                    etc_php_subdirs.push(path.join(CONF_D));
                }
            }
        })?;
    }

    Ok(
        etc_php_dirs.into_iter()
            .filter(|path| path.ends_with(CONF_D))
            .chain(etc_php_subdirs.into_iter())
            .collect()
    )
}

#[cfg(feature="containers")]
fn ask_php_for_conf_d(ctx: &mut Context) -> Result<PathBuf, String> {
    let runtime_exe = ctx.composer_settings
        .runtime_exe
        .clone()
        .unwrap_or(DEFAULT_RUNTIME.to_owned());

    let args = [runtime_exe, "--ini".to_owned()];
    let output = capture_command(ctx, &args, &[])
        .and_then(|out| String::from_utf8(out.stdout)
            .map_err(|e| format!("Error parsing command output: {}", e)))
        .map_err(|e| format!("Error reading command output: {}", e))?;

    // match any line that ends with /etc/php*/**/conf.d, get first result
    let re = Regex::new(r#"(?m).*?(/etc/php\d/.*?conf.d)$"#).expect("Invalid regex");

    let conf_d = re.captures(&output)
        .and_then(|cap| cap.get(1))
        .ok_or("PHP configuration directory was not found".to_owned())?
        .as_str();

    Ok(PathBuf::from(conf_d))
}

#[cfg(feature="containers")]
pub fn finish(ctx: &mut Context) -> Result<(), StepError> {
    list_packages(ctx)
        .map_err(|e| warn!("Can't list composer packages: {}", e)).ok();
    if !ctx.composer_settings.keep_composer {
        fs::remove_file(Path::new("/vagga/root/usr/local/bin/composer"))
            .map_err(|e| format!("Error removing '/usr/local/bin/composer': {}", e))?;
    }

    Ok(())
}

#[cfg(feature="containers")]
fn list_packages(ctx: &mut Context) -> Result<(), StepError> {
    let mut cmd = composer_cmd(ctx)?;
    cmd.arg("show");

    capture_output(cmd)
        .and_then(|out| {
            File::create("/vagga/container/composer-list.txt")
            .and_then(|mut f| f.write_all(&out.stdout))
            .map_err(|e| format!("Error dumping composer package list: {}", e))
        })?;

    Ok(())
}

impl BuildStep for ComposerConfig {
    fn name(&self) -> &'static str { "ComposerConfig" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.opt_field("runtime_exe", &self.runtime_exe);
        hash.opt_field("include_path", &self.include_path);
        hash.field("install_runtime", self.install_runtime);
        hash.field("install_dev", self.install_dev);
        hash.field("keep_composer", self.keep_composer);
        hash.opt_field("vendor_dir", &self.vendor_dir);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, _build: bool)
        -> Result<(), StepError>
    {
        guard.ctx.composer_settings = self.clone();
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for ComposerInstall {
    fn name(&self) -> &'static str { "ComposerInstall" }
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
            composer_install(&mut guard.distro, &mut guard.ctx, &self.0)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

fn get<'x>(dic: &'x Json, key: &str) -> &'x Json {
    // TODO(tailhook) is there a better way for the following?
    let x: &'static Json = unsafe { ::std::mem::transmute(&Json::Null) };
    dic.get(key).unwrap_or(x)
}

fn hash_lock_file(path: &Path, hash: &mut Digest) -> Result<(), VersionError> {
    File::open(&path).map_err(|e| VersionError::io(e, path))
    .and_then(|mut f| from_reader(&mut f)
        .map_err(|e| format_err!("bad json in {:?}: {}", path, e).into()))
    .and_then(|data: Json| {
        let packages = data.get("packages")
            .ok_or("Missing 'packages' property from composer.lock".to_owned())?;
        let packages = packages.as_array()
            .ok_or("'packages' property is not an array".to_owned())?;
        for package in packages.iter() {
            for key in LOCKFILE_RELEVANT_KEYS.iter() {
                hash.field(key, get(&package, key));
            }
            hash.field("require", get(&package, "require"));
            hash.field("require-dev", get(&package, "require-dev"));
        }
        Ok(())
    })
}

impl BuildStep for ComposerDependencies {
    fn name(&self) -> &'static str { "ComposerDependencies" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        let base_path: PathBuf = {
            let path = Path::new("/work");
            if let Some(ref working_dir) = self.working_dir {
                path.join(working_dir)
            } else {
                path.to_owned()
            }
        };

        let path = base_path.join("composer.lock");
        if path.exists() {
            hash_lock_file(&path, hash)?;
        }

        let path = base_path.join("composer.json");
        File::open(&path).map_err(|e| VersionError::io(e, &path))
        .and_then(|mut f| from_reader(&mut f)
            .map_err(|e| format_err!("bad json in {:?}: {}", &path, e).into()))
        .map(|data| {
            // Jsons are sorted so should be hash as string predictably
            hash.field("require", get(&data, "require"));
            hash.field("conflict", get(&data, "conflict"));
            hash.field("replace", get(&data, "replace"));
            hash.field("provide", get(&data, "provide"));
            hash.field("autoload", get(&data, "autoload"));
            hash.field("repositories", get(&data, "repositories"));
            hash.field("minimum-stability", get(&data, "minimum-stability"));
            hash.field("prefer-stable", get(&data, "prefer-stable"));
            hash.field("require-dev", get(&data, "autoload-dev"));
        })
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        configure(&mut guard.ctx)?;
        if build {
            composer_dependencies(&mut guard.distro, &mut guard.ctx, &self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
