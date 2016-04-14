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
pub struct GemInstall(Vec<String>);

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
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

#[derive(Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct GemConfig {
    pub install_ruby: bool,
    pub gem_exe: Option<String>,
    pub update_gem: bool,
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
    -> Result<Step, E>
{
    val.map(|x| Step(Rc::new(x) as Rc<BuildStep>))
}

fn decode_step<D: Decoder>(options: &[&str], index: usize, d: &mut D)
    -> Result<Step, D::Error>
{
    match options[index] {
        "Alpine" => step(cmd::alpine::Alpine::decode(d)),
        "Ubuntu" => step(cmd::ubuntu::Ubuntu::decode(d)),
        "UbuntuRepo" => step(cmd::ubuntu::UbuntuRepo::decode(d)),
        "UbuntuRelease" => step(cmd::ubuntu::UbuntuRelease::decode(d)),
        "UbuntuPPA" => step(cmd::ubuntu::UbuntuPPA::decode(d)),
        "UbuntuUniverse" => step(cmd::ubuntu::UbuntuUniverse::decode(d)),
        "AptTrust" => step(cmd::ubuntu::AptTrust::decode(d)),
        "Install" => step(cmd::packaging::Install::decode(d)),
        "BuildDeps" => step(cmd::packaging::BuildDeps::decode(d)),
        "PipConfig" => step(cmd::pip::PipConfig::decode(d)),
        "Py2Install" => step(cmd::pip::Py2Install::decode(d)),
        "Py2Requirements" => step(cmd::pip::Py2Requirements::decode(d)),
        "Py3Install" => step(cmd::pip::Py3Install::decode(d)),
        "Py3Requirements" => step(cmd::pip::Py3Requirements::decode(d)),
        "Tar" => step(cmd::tarcmd::Tar::decode(d)),
        "TarInstall" => step(cmd::tarcmd::TarInstall::decode(d)),
        "Sh" => step(cmd::generic::Sh::decode(d)),
        "Cmd" => step(cmd::generic::Cmd::decode(d)),
        "Env" => step(cmd::generic::Env::decode(d)),
        "Text" => step(cmd::text::Text::decode(d)),
        "EnsureDir" => step(cmd::dirs::EnsureDir::decode(d)),
        "CacheDirs" => step(cmd::dirs::CacheDirs::decode(d)),
        "EmptyDir" => step(cmd::dirs::EmptyDir::decode(d)),
        "Remove" => step(cmd::dirs::Remove::decode(d)),
        "Depends" => step(cmd::generic::Depends::decode(d)),
        "Container" => step(cmd::subcontainer::Container::decode(d)),
        "Build" => step(cmd::subcontainer::Build::decode(d)),
        "SubConfig" => step(cmd::subcontainer::SubConfig::decode(d)),
        "NpmConfig" => step(cmd::npm::NpmConfig::decode(d)),
        "NpmDependencies" => step(cmd::npm::NpmDependencies::decode(d)),
        "NpmInstall" => step(cmd::npm::NpmInstall::decode(d)),
        step_name => panic!("Step {} is not yet implemented", step_name),
    }
}

impl Decodable for Step {
    fn decode<D: Decoder>(d: &mut D) -> Result<Step, D::Error> {
        // TODO(tailhook) this is just too slow
        //                move it to lazy_static
        let val = builder_validator();
        let options = val.options.iter().map(|&(ref x, _)| &x[..])
            .collect::<Vec<_>>();
        Ok(try!(d.read_enum("BuildStep", |d| {
            d.read_enum_variant(&options, |d, index| {
                decode_step(&options, index, d)
            })
        })))
    }
}
