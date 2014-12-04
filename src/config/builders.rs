use std::fmt::{Show, Formatter, FormatError};
use std::default::Default;

use quire::validate as V;

#[deriving(Decodable, Show)]
pub struct DebianRepo {
    url: String,
    suite: String,
    components: Vec<String>,
}

#[deriving(Decodable, Show)]
pub struct AptKey {
    key_server: String,
    keys: Vec<String>,
}

#[deriving(Decodable, Show)]
pub struct PacmanRepo {
  name: String,
  url: String,
}

#[deriving(Decodable)]
pub struct TarInfo {
    url: String,
    sha256: String,
    path: Path,
}

#[deriving(Decodable)]
pub struct FileInfo {
    name: Path,
    contents: String,
}

#[deriving(Decodable)]
pub enum Builder {
    // Generic
    Sh(String),
    Cmd(String),
    Depend(Path),
    Tar(TarInfo),
    AddFile(FileInfo),
    Remove(Path),
    EnsureDir(Path),
    EmptyDir(Path),
    Busybox,

    // Ubuntu
    UbuntuCore(String),
    AddUbuntuPPA(String),

    // Ubuntu/Debian
    AptGetInstall(Vec<String>),
    AddDebianRepo(DebianRepo),
    AddAptKey(AptKey),

    // Arch
    ArchBase,
    PacmanInstall(Vec<String>),
    PacmanRemove(Vec<String>),
    PacmanBuild(Path),
    AddPacmanRepo(PacmanRepo),

    // Alpine
    AlpineInstall(Vec<String>),
    AlpineRemove(Vec<String>),

    // Docker
    DockerImage(String),
    DockerPrivate(String),
    Dockerfile(Path),

    // Languages
    NpmInstall(Vec<String>),
    PipRequirement(Path),
    PipInstall(Vec<String>),
    GemInstall(Vec<String>),
}

pub fn builder_validator<'x>() -> Box<V::Validator + 'x> {
    return box V::Enum { options: vec!(
        ("UbuntuCore".to_string(), box V::Scalar {
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
            _ => unimplemented!(),
        }
        return Ok(());
    }
}

