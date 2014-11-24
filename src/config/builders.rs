pub enum Builder {
    // Generic
    Sh(String),
    Cmd(String),
    Depend(Path),
    Tar { url: String, sha256: String, path: Path },
    AddFile { name: Path, contents: String },
    Remove(Path),
    EnsureDir(Path),
    EmptyDir(Path),
    Busybox,

    // Ubuntu
    UbuntuBase(String),
    AddUbuntuPPA(String),

    // Ubuntu/Debian
    AptGetInstall(Vec<String>),
    AddDebianRepo { url: String, suite: String, components: Vec<String> },
    AddAptKey { key_server: String, keys: Vec<String> },

    // Arch
    ArchBase,
    PacmanInstall(Vec<String>),
    PacmanRemove(Vec<String>),
    PacmanBuild(Path),
    AddPacmanRepo { name: String, url: String },

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
