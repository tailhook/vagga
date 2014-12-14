use std::fmt::{Show, Formatter, FormatError};
use std::default::Default;
use std::collections::TreeMap;

use quire::validate as V;

#[deriving(Decodable, Show, Clone)]
pub struct DebianRepo {
    pub url: String,
    pub suite: String,
    pub components: Vec<String>,
}

#[deriving(Decodable, Show, Clone)]
pub struct AptKey {
    pub key_server: String,
    pub keys: Vec<String>,
}

#[deriving(Decodable, Show, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
}

#[deriving(Decodable, Clone)]
pub struct TarInfo {
    pub url: String,
    pub sha256: String,
    pub path: Path,
}

#[deriving(Decodable, Clone)]
pub struct FileInfo {
    pub name: Path,
    pub contents: String,
}

#[deriving(Decodable, Clone)]
pub enum Builder {
    // -- Generic --
    Sh(String),
    Cmd(Vec<String>),
    Env(TreeMap<String, String>),
    Depends(Path),
    //Tar(TarInfo),
    //AddFile(FileInfo),
    Remove(Path),
    EnsureDir(Path),
    EmptyDir(Path),
    //Busybox,

    // -- Ubuntu --
    UbuntuCore(String),
    //AddUbuntuPPA(String),

    // -- Ubuntu/Debian --
    //AptGetInstall(Vec<String>),
    //AddDebianRepo(DebianRepo),
    //AddAptKey(AptKey),

    // -- Arch --
    //ArchBase,
    //PacmanInstall(Vec<String>),
    //PacmanRemove(Vec<String>),
    //PacmanBuild(Path),
    //AddPacmanRepo(PacmanRepo),

    // -- Alpine --
    //AlpineInstall(Vec<String>),
    //AlpineRemove(Vec<String>),

    // -- Docker --
    //DockerImage(String),
    //DockerPrivate(String),
    //Dockerfile(Path),

    // -- Languages --
    //NpmInstall(Vec<String>),
    //PipRequirement(Path),
    //PipInstall(Vec<String>),
    //GemInstall(Vec<String>),
}

pub fn builder_validator<'x>() -> Box<V::Validator + 'x> {
    return box V::Enum { options: vec!(
        ("UbuntuCore".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),
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
        ("Env".to_string(), box V::Mapping {
            key_element: box V::Scalar {
                .. Default::default() } as Box<V::Validator>,
            value_element: box V::Scalar {
                .. Default::default() } as Box<V::Validator>,
        .. Default::default() } as Box<V::Validator>),
        ("Depends".to_string(), box V::Scalar {
        .. Default::default() } as Box<V::Validator>),
    ), .. Default::default() } as Box<V::Validator>;
}

impl Show for Builder {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FormatError> {
        match self {
            &UbuntuCore(ref name) => {
                try!("!UbuntuCore ".fmt(fmt));
                try!(name.fmt(fmt));
            }
            &Cmd(ref command) => {
                try!("!Sh ".fmt(fmt));
                try!(command.fmt(fmt));
            }
            &Sh(ref command) => {
                try!("!Sh ".fmt(fmt));
                try!(command.fmt(fmt));
            }
            &Env(ref map) => {
                try!("!Env {".fmt(fmt));
                for (k, v) in map.iter() {
                    try!(k.fmt(fmt));
                    try!(": ".fmt(fmt));
                    try!(v.fmt(fmt));
                    try!(", ".fmt(fmt));
                }
                try!("}".fmt(fmt));
            }
            &Remove(ref path) => {
                try!("!Remove ".fmt(fmt));
                try!(path.display().fmt(fmt));
            }
            &EmptyDir(ref path) => {
                try!("!EmptyDir ".fmt(fmt));
                try!(path.display().fmt(fmt));
            }
            &EnsureDir(ref path) => {
                try!("!EnsureDir ".fmt(fmt));
                try!(path.display().fmt(fmt));
            }
            &Depends(ref path) => {
                try!("!Depends ".fmt(fmt));
                try!(path.display().fmt(fmt));
            }
        }
        return Ok(());
    }
}

