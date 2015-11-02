use std::path::PathBuf;
use std::default::Default;
use std::collections::BTreeMap;

use quire::validate as V;

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct DebianRepo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct AptKey {
    pub key_server: String,
    pub keys: Vec<String>,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct TarInfo {
    pub url: String,
    pub sha256: Option<String>,
    pub path: PathBuf,
    pub subdir: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct TarInstallInfo {
    pub url: String,
    pub sha256: Option<String>,
    pub subdir: Option<PathBuf>,
    pub script: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct GitInfo {
    pub url: String,
    pub revision: Option<String>,
    pub branch: Option<String>,
    pub path: PathBuf,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct GitInstallInfo {
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
pub struct UbuntuReleaseInfo {
    pub version: String,
    pub keep_chfn_command: bool,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct UbuntuRepoInfo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(Default, Clone, RustcDecodable, Debug, RustcEncodable)]
pub struct PipSettings {
    pub find_links: Vec<String>,
    pub index_urls: Vec<String>,
    pub trusted_hosts: Vec<String>,
    pub dependencies: bool,
    pub cache_wheels: bool,
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
pub struct SubConfigInfo {
    pub source: Source,
    pub path: PathBuf,
    pub container: String,
    pub cache: Option<bool>,
    pub change_dir: Option<bool>,
}


#[derive(RustcEncodable, RustcDecodable, Clone, Debug)]
pub enum Builder {
    // -- Generic --
    Sh(String),
    Cmd(Vec<String>),
    Env(BTreeMap<String, String>),
    Depends(PathBuf),
    Tar(TarInfo),
    TarInstall(TarInstallInfo),
    Git(GitInfo),
    GitInstall(GitInstallInfo),
    Text(BTreeMap<PathBuf, String>),
    //AddFile(FileInfo),
    Remove(PathBuf),
    EnsureDir(PathBuf),
    EmptyDir(PathBuf),
    CacheDirs(BTreeMap<PathBuf, String>),
    //Busybox,

    // -- Generic --
    Install(Vec<String>),
    BuildDeps(Vec<String>),
    Container(String),
    SubConfig(SubConfigInfo),

    // -- Ubuntu --
    Ubuntu(String),
    UbuntuRelease(UbuntuReleaseInfo),
    UbuntuRepo(UbuntuRepoInfo),
    UbuntuUniverse,
    //AddUbuntuPPA(String),

    // -- Ubuntu/Debian --
    //AddDebianRepo(DebianRepo),
    //AddAptKey(AptKey),

    // -- Arch --
    //ArchBase,
    //PacmanInstall(Vec<String>),
    //PacmanRemove(Vec<String>),
    //PacmanBuild(Path),
    //AddPacmanRepo(PacmanRepo),

    // -- Alpine --
    Alpine(String),
    //AlpineRemove(Vec<String>),

    // -- Docker --
    //DockerImage(String),
    //DockerPrivate(String),
    //Dockerfile(Path),

    // -- Languages --
    NpmInstall(Vec<String>),
    //PipRequirement(Path),
    //GemInstall(Vec<String>),

    // -- Python --
    PipConfig(PipSettings),
    Py2Install(Vec<String>),
    Py2Requirements(PathBuf),
    Py3Install(Vec<String>),
    Py3Requirements(PathBuf),
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
    .option("Text", V::Mapping::new(
        V::Directory::new().is_absolute(true),
        V::Scalar::new()))
    .option("Ubuntu", V::Scalar::new())
    .option("UbuntuRelease", V::Structure::new()
        .member("version", V::Scalar::new())
        .member("keep_chfn_command", V::Scalar::new().default(false)))
    .option("UbuntuRepo", V::Structure::new()
        .member("url", V::Scalar::new())
        .member("suite", V::Scalar::new())
        .member("components", V::Sequence::new(V::Scalar::new())))
    .option("UbuntuUniverse", V::Nothing)
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
    .option("Alpine", V::Scalar::new())

    // Python
    .option("PipConfig", V::Structure::new()
        .member("dependencies", V::Scalar::new().default(false).optional())
        .member("cache_wheels", V::Scalar::new().default(true))
        .member("find_links", V::Sequence::new(V::Scalar::new()))
        .member("index_urls", V::Sequence::new(V::Scalar::new()))
        .member("trusted_hosts", V::Sequence::new(V::Scalar::new())))
    .option("Py2Install", V::Sequence::new(V::Scalar::new()))
    .option("Py2Requirements", V::Scalar::new()
        .default("requirements.txt"))
    .option("Py3Install", V::Sequence::new(V::Scalar::new()))
    .option("Py3Requirements",
        V::Scalar::new().default("requirements.txt"))

    // Node.js
    .option("NpmInstall", V::Sequence::new(V::Scalar::new()))
}
