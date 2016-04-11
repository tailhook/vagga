use std::rc::Rc;
use std::path::PathBuf;
use std::default::Default;
use std::collections::BTreeMap;

use quire::validate as V;
use libc::{uid_t, gid_t};
use rustc_serialize::{Decodable, Decoder};
use builder::commands as cmd;

use build_step::{Step, BuildStep};

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Sh(String);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Cmd(Vec<String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Env(BTreeMap<String, String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Depends(PathBuf);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Text(BTreeMap<PathBuf, String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Remove(PathBuf);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct EnsureDir(PathBuf);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct EmptyDir(PathBuf);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct CacheDirs(BTreeMap<PathBuf, String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct BuildDeps(Vec<String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Container(String);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Ubuntu(String);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct UbuntuPPA(String);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct GemInstall(Vec<String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct DebianRepo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct AptTrust {
    pub server: Option<String>,
    pub keys: Vec<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Tar {
    pub url: String,
    pub sha256: Option<String>,
    pub path: PathBuf,
    pub subdir: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct TarInstall {
    pub url: String,
    pub sha256: Option<String>,
    pub subdir: Option<PathBuf>,
    pub script: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Git {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
    pub path: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct GitInstall {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
    pub subdir: PathBuf,
    pub script: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct FileInfo {
    pub name: PathBuf,
    pub contents: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct UbuntuRelease {
    pub version: String,
    pub arch: String,
    pub keep_chfn_command: bool,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct UbuntuRepo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct PipConfig {
    pub find_links: Vec<String>,
    pub index_urls: Vec<String>,
    pub trusted_hosts: Vec<String>,
    pub dependencies: bool,
    pub cache_wheels: bool,
    pub install_python: bool,
    pub python_exe: Option<String>,
}


#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct Py2Install(Vec<String>);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct Py2Requirements(PathBuf);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct Py3Install(Vec<String>);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct Py3Requirements(PathBuf);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct NpmConfig {
    pub install_node: bool,
    pub npm_exe: String,
}

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct NpmInstall(Vec<String>);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct ComposerConfig {
    // It is used 'runtime' instead of 'php' in order to support hhvm in the future
    pub install_runtime: bool,
    pub install_dev: bool,
    pub runtime_exe: Option<String>,
    pub include_path: Option<String>,
}
#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct ComposerInstall(Vec<String>);

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct GemConfig {
    pub install_ruby: bool,
    pub gem_exe: Option<String>,
    pub update_gem: bool,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct GitSource {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub enum Source {
    Git(GitSource),
    Container(String),
    Directory,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct SubConfig {
    pub source: Source,
    pub path: PathBuf,
    pub container: String,
    pub cache: Option<bool>,
    pub change_dir: Option<bool>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct Build {
    pub container: String,
    pub source: PathBuf,
    pub path: Option<PathBuf>,
    pub temporary_mount: Option<PathBuf>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct PyFreeze {
    pub freeze_file: PathBuf,
    pub requirements: Option<PathBuf>,
    pub packages: Vec<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub mode: u32,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct Copy {
    pub source: PathBuf,
    pub path: PathBuf,
    pub owner_uid: Option<uid_t>,
    pub owner_gid: Option<gid_t>,
    pub ignore_regex: String,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct NpmDependencies {
    pub file: PathBuf,
    pub package: bool,
    pub dev: bool,
    pub peer: bool,
    pub bundled: bool,
    pub optional: bool,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
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

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct GemBundle {
    pub gemfile: PathBuf,
    pub without: Vec<String>,
    pub trust_policy: Option<String>,
}

pub fn builder_validator<'x>() -> V::Enum<'x> {
    V::Enum::new()
    .option("Install", V::Sequence::new(V::Scalar::new()))
    .option("BuildDeps", V::Sequence::new(V::Scalar::new()))
    .option("Container", V::Scalar::new())
    .option("SubConfig", V::Structure::new()
        .member("source", V::Enum::new()
            .option("Directory", V::Nothing)
            .option("Container", V::Scalar::new())
            .option("Git", V::Structure::new()
                .member("url", V::Scalar::new())
                .member("revision", V::Scalar::new().optional())
                .member("branch", V::Scalar::new().optional()))
            .optional()
            .default_tag("Directory"))
        .member("path".to_string(), V::Directory::new()
            .is_absolute(false)
            .default("vagga.yaml"))
        .member("container", V::Scalar::new())
        .member("cache", V::Scalar::new().optional())
        .member("change_dir", V::Scalar::new().optional()))
    .option("Build", V::Structure::new()
        .member("container", V::Scalar::new())
        .member("source".to_string(),
            V::Directory::new().is_absolute(true).default("/"))
        .member("path".to_string(),
            V::Directory::new().is_absolute(true).optional())
        .member("temporary_mount".to_string(),
            V::Directory::new().is_absolute(true).optional()))
    .option("Text", V::Mapping::new(
        V::Directory::new().is_absolute(true),
        V::Scalar::new()))
    .option("Copy", V::Structure::new()
        .member("source", V::Scalar::new())
        .member("path", V::Directory::new().is_absolute(true))
        .member("ignore_regex", V::Scalar::new().default(
            r#"(^|/)\.(git|hg|svn|vagga)($|/)|~$|\.bak$|\.orig$|^#.*#$"#))
        .member("owner_uid", V::Numeric::new().min(0).optional())
        .member("owner_gid", V::Numeric::new().min(0).optional()))

    .option("Ubuntu", V::Scalar::new())
    .option("UbuntuRelease", V::Structure::new()
        .member("version", V::Scalar::new())
        .member("arch", V::Scalar::new().default("amd64"))
        .member("keep_chfn_command", V::Scalar::new().default(false)))
    .option("UbuntuRepo", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("suite", V::Scalar::new())
        .member("components", V::Sequence::new(V::Scalar::new())))
    .option("UbuntuPPA", V::Scalar::new())
    .option("UbuntuUniverse", V::Nothing)
    .option("AptTrust", V::Structure::new()
        .member("server", V::Scalar::new().optional())
        .member("keys", V::Sequence::new(V::Scalar::new())))
    .option("Sh", V::Scalar::new())
    .option("Cmd", V::Sequence::new(V::Scalar::new()))
    .option("Remove", V::Directory::new().is_absolute(true))
    .option("EnsureDir", V::Directory::new().is_absolute(true))
    .option("EmptyDir", V::Directory::new().is_absolute(true))
    .option("CacheDirs", V::Mapping::new(
        V::Directory::new().is_absolute(true),
        V::Scalar::new()))
    .option("Env", V::Mapping::new(
        V::Scalar::new(),
        V::Scalar::new()))
    .option("Depends", V::Scalar::new())
    .option("Git", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("revision", V::Scalar::new().optional())
        .member("branch", V::Scalar::new().optional())
        .member("path", V::Directory::new().is_absolute(true)))
    .option("GitInstall", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("revision", V::Scalar::new().optional())
        .member("branch", V::Scalar::new().optional())
        .member("subdir", V::Directory::new()
            .default(".").is_absolute(false))
        .member("script", V::Scalar::new()
                .default("./configure --prefix=/usr\n\
                          make\n\
                          make install\n")))
    .option("Tar", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("sha256", V::Scalar::new().optional())
        .member("path", V::Directory::new().is_absolute(true).default("/"))
        .member("subdir", V::Directory::new().default("").is_absolute(false)))
    .option("TarInstall", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("sha256", V::Scalar::new().optional())
        .member("subdir", V::Directory::new().optional().is_absolute(false))
        .member("script", V::Scalar::new()
                .default("./configure --prefix=/usr\n\
                          make\n\
                          make install\n")))
    .option("Download", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("path", V::Directory::new().is_absolute(true))
        .member("mode", V::Numeric::new().default(0o644).min(0).max(0o1777)))
    .option("Alpine", V::Scalar::new())

    // Python
    .option("PipConfig", V::Structure::new()
        .member("dependencies", V::Scalar::new().default(false).optional())
        .member("cache_wheels", V::Scalar::new().default(true))
        .member("find_links", V::Sequence::new(V::Scalar::new()))
        .member("index_urls", V::Sequence::new(V::Scalar::new()))
        .member("trusted_hosts", V::Sequence::new(V::Scalar::new()))
        .member("python_exe", V::Scalar::new().optional())
        .member("install_python", V::Scalar::new().default(true)))
    .option("Py2Install", V::Sequence::new(V::Scalar::new()))
    .option("Py2Requirements", V::Scalar::new()
        .default("requirements.txt"))
    .option("Py3Install", V::Sequence::new(V::Scalar::new()))
    .option("Py3Requirements",
        V::Scalar::new().default("requirements.txt"))
    .option("PyFreeze", V::Structure::new()
        .member("freeze_file", V::Scalar::new().default("requirements.txt"))
        .member("requirements", V::Scalar::new().optional())
        .member("packages", V::Sequence::new(V::Scalar::new())))

    // Node.js
    .option("NpmConfig", V::Structure::new()
        .member("npm_exe", V::Scalar::new().default("npm"))
        .member("install_node", V::Scalar::new().default(true)))
    .option("NpmInstall", V::Sequence::new(V::Scalar::new()))
    .option("NpmDependencies", V::Structure::new()
        .member("file", V::Scalar::new().default("package.json"))
        .member("package", V::Scalar::new().default(true))
        .member("dev", V::Scalar::new().default(true))
        .member("peer", V::Scalar::new().default(false))
        .member("bundled", V::Scalar::new().default(true))
        .member("optional", V::Scalar::new().default(false)))

    // Composer
    .option("ComposerConfig", V::Structure::new()
        .member("install_runtime", V::Scalar::new().default(true))
        .member("install_dev", V::Scalar::new().default(false))
        .member("runtime_exe", V::Scalar::new().optional())
        .member("include_path", V::Scalar::new().optional()))
    .option("ComposerInstall", V::Sequence::new(V::Scalar::new()))
    .option("ComposerDependencies", V::Structure::new()
        .member("working_dir", V::Scalar::new().optional())
        .member("dev", V::Scalar::new().default(true))
        .member("prefer", V::Scalar::new().optional())
        .member("ignore_platform_reqs", V::Scalar::new().default(false))
        .member("no_autoloader", V::Scalar::new().default(false))
        .member("no_scripts", V::Scalar::new().default(false))
        .member("no_plugins", V::Scalar::new().default(false))
        .member("optimize_autoloader", V::Scalar::new().default(false))
        .member("classmap_authoritative", V::Scalar::new().default(false)))

    // Ruby
    .option("GemConfig", V::Structure::new()
        .member("install_ruby", V::Scalar::new().default(true))
        .member("gem_exe", V::Scalar::new().optional())
        .member("update_gem", V::Scalar::new().default(true)))
    .option("GemInstall", V::Sequence::new(V::Scalar::new()))
    .option("GemBundle", V::Structure::new()
        .member("gemfile", V::Scalar::new().default("Gemfile"))
        .member("without", V::Sequence::new(V::Scalar::new()))
        .member("trust_policy", V::Scalar::new().optional()))
}

fn step<T: BuildStep + 'static, E>(val: Result<T, E>)
    -> Result<Rc<BuildStep>, E>
{
    val.map(|x| Rc::new(x) as Rc<BuildStep>)
}

impl Decodable for Step {
    fn decode<D: Decoder>(d: &mut D) -> Result<Step, D::Error> {
        // TODO(tailhook) this is just too slow
        //                move it to lazy_static
        let val = builder_validator();
        let options = val.options.iter().map(|&(ref x, _)| &x[..])
            .collect::<Vec<_>>();
        Ok(Step(try!(d.read_enum("BuildStep", |d| {
            d.read_enum_variant(&options, |d, index| {
                match options[index] {
                    "Alpine" => step(cmd::alpine::Alpine::decode(d)),
                    "Install" => step(cmd::packaging::Install::decode(d)),
                    step_name => panic!("Step {} is not yet implemented",
                                        step_name),
                }
            })
        }))))
    }
}
