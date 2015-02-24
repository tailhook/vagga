use std::fmt::{Debug, Formatter};
use std::fmt::Error as FormatError;
use std::default::Default;
use std::collections::BTreeMap;

use quire::validate as V;
use serialize::json;

#[derive(Encodable, Decodable, Show, Clone)]
pub struct DebianRepo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(Encodable, Decodable, Show, Clone)]
pub struct AptKey {
    pub key_server: String,
    pub keys: Vec<String>,
}

#[derive(Encodable, Decodable, Show, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
}

#[derive(Encodable, Decodable, Clone)]
pub struct TarInfo {
    pub url: String,
    pub sha256: Option<String>,
    pub path: Path,
    pub subdir: Path,
}

#[derive(Encodable, Decodable, Clone)]
pub struct TarInstallInfo {
    pub url: String,
    pub sha256: Option<String>,
    pub subdir: Option<Path>,
    pub script: String,
}

#[derive(Encodable, Decodable, Clone)]
pub struct FileInfo {
    pub name: Path,
    pub contents: String,
}

#[derive(Encodable, Decodable, Clone)]
pub struct UbuntuRepoInfo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[derive(Default, Clone, Decodable, Encodable)]
pub struct PipSettings {
    pub find_links: Vec<String>,
    pub index_urls: Vec<String>,
    pub dependencies: bool,
}


#[derive(Encodable, Decodable, Clone)]
pub enum Builder {
    // -- Generic --
    Sh(String),
    Cmd(Vec<String>),
    Env(BTreeMap<String, String>),
    Depends(Path),
    Tar(TarInfo),
    TarInstall(TarInstallInfo),
    //AddFile(FileInfo),
    Remove(Path),
    EnsureDir(Path),
    EmptyDir(Path),
    CacheDirs(BTreeMap<Path, String>),
    //Busybox,

    // -- Generic --
    Install(Vec<String>),
    BuildDeps(Vec<String>),
    Container(String),

    // -- Ubuntu --
    Ubuntu(String),
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
    Py2Requirements(Path),
    Py3Install(Vec<String>),
    Py3Requirements(Path),
}

pub fn builder_validator<'x>() -> Box<V::Validator + 'x> {
    return box V::Enum { options: vec!(
        ("Install".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("BuildDeps".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Container".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),

        ("Ubuntu".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),
        ("UbuntuRepo".to_string(), box V::Structure {
            members: vec!(
                ("url".to_string(), box V::Scalar {
                    .. Default::default() } as Box<V::Validator>),
                ("suite".to_string(), box V::Scalar {
                    .. Default::default() } as Box<V::Validator>),
                ("components".to_string(), box V::Sequence {
                    element: box V::Scalar {
                        .. Default::default() } as Box<V::Validator>,
                    .. Default::default() } as Box<V::Validator>),
            ),
        .. Default::default() } as Box<V::Validator>),
        ("UbuntuUniverse".to_string(), box V::Nothing as Box<V::Validator>),
        ("Sh".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),
        ("Cmd".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Remove".to_string(), box V::Directory {
            absolute: Some(true),
        .. Default::default() } as Box<V::Validator>),
        ("EnsureDir".to_string(), box V::Directory {
            absolute: Some(true),
        .. Default::default() } as Box<V::Validator>),
        ("EmptyDir".to_string(), box V::Directory {
            absolute: Some(true),
        .. Default::default() } as Box<V::Validator>),
        ("CacheDirs".to_string(), box V::Mapping {
            key_element: box V::Directory {
                absolute: Some(true),
                .. Default::default() } as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Env".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default() } as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Depends".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),
        ("Tar".to_string(), box V::Structure {
            members: vec!(
                ("url".to_string(), box V::Scalar {
                    .. Default::default() } as Box<V::Validator>),
                ("sha256".to_string(), box V::Scalar {
                    optional: true,
                    .. Default::default() } as Box<V::Validator>),
                ("path".to_string(), box V::Directory {
                    default: Some(Path::new("/")),
                    .. Default::default() } as Box<V::Validator>),
                ("subdir".to_string(), box V::Directory {
                    default: Some(Path::new("")),
                    absolute: Some(false),
                    .. Default::default() } as Box<V::Validator>),
            ),
        .. Default::default() } as Box<V::Validator>),
        ("TarInstall".to_string(), box V::Structure {
            members: vec!(
                ("url".to_string(), box V::Scalar {
                    .. Default::default() } as Box<V::Validator>),
                ("sha256".to_string(), box V::Scalar {
                    optional: true,
                    .. Default::default() } as Box<V::Validator>),
                ("subdir".to_string(), box V::Directory {
                    optional: true,
                    absolute: Some(false),
                    .. Default::default() } as Box<V::Validator>),
                ("script".to_string(), box V::Scalar {
                    default: Some("./configure --prefix=/usr\n\
                                   make\n\
                                   make install\n\
                                   ".to_string()),
                    .. Default::default() } as Box<V::Validator>),
            ),
        .. Default::default() } as Box<V::Validator>),

        ("Alpine".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),

        // Python
        ("PipConfig".to_string(), box V::Structure {
            members: vec!(
                ("dependencies".to_string(), box V::Scalar {
                    default: Some("false".to_string()),
                    optional: true,
                    .. Default::default() } as Box<V::Validator>),
                ("find_links".to_string(), box V::Sequence {
                    element: box V::Scalar {
                        .. Default::default() } as Box<V::Validator>,
                    .. Default::default() } as Box<V::Validator>),
                ("index_urls".to_string(), box V::Sequence {
                    element: box V::Scalar {
                        .. Default::default() } as Box<V::Validator>,
                    .. Default::default() } as Box<V::Validator>),
            ),
        .. Default::default() } as Box<V::Validator>),
        ("Py2Install".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Py2Requirements".to_string(), box V::Scalar {
            default: Some("requirements.txt".to_string()),
        .. Default::default() } as Box<V::Validator>),
        ("Py3Install".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Py3Requirements".to_string(), box V::Scalar {
            default: Some("requirements.txt".to_string()),
        .. Default::default() } as Box<V::Validator>),

        // Node.js
        ("NpmInstall".to_string(), box V::Sequence {
            element: box V::Scalar {
            .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),

    ), .. Default::default() } as Box<V::Validator>;
}

impl Debug for Builder {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        write!(fmt, "{}", json::as_json(self))
    }
}

