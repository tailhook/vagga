use std::fmt;
use std::rc::Rc;

use builder::commands as cmd;
use quire::validate as V;
use serde::de::{self, Deserializer, Deserialize};
use serde::de::{VariantAccess, Visitor, EnumAccess};

use build_step::{Step, BuildStep};

const COMMANDS: &'static [&'static str] = &[
    "Alpine",
    "AlpineRepo",
    "Ubuntu",
    "UbuntuRepo",
    "UbuntuRelease",
    "UbuntuPPA",
    "UbuntuUniverse",
    "AptTrust",
    "Repo",
    "Install",
    "BuildDeps",
    "Git",
    "GitInstall",
    "GitDescribe",
    "PipConfig",
    "Py2Install",
    "Py2Requirements",
    "Py3Install",
    "Py3Requirements",
    "Tar",
    "TarInstall",
    "Unzip",
    "Sh",
    "Cmd",
    "RunAs",
    "Env",
    "Text",
    "Copy",
    "Download",
    "EnsureDir",
    "CacheDirs",
    "EmptyDir",
    "Remove",
    "Depends",
    "Container",
    "Build",
    "SubConfig",
    "NpmConfig",
    "NpmDependencies",
    "YarnDependencies",
    "NpmInstall",
    "GemInstall",
    "GemBundle",
    "GemConfig",
    "ComposerInstall",
    "ComposerDependencies",
    "ComposerConfig",
];

pub enum CommandName {
    Alpine,
    AlpineRepo,
    Ubuntu,
    UbuntuRepo,
    UbuntuRelease,
    UbuntuPPA,
    UbuntuUniverse,
    AptTrust,
    Repo,
    Install,
    BuildDeps,
    Git,
    GitInstall,
    GitDescribe,
    PipConfig,
    Py2Install,
    Py2Requirements,
    Py3Install,
    Py3Requirements,
    Tar,
    TarInstall,
    Unzip,
    Sh,
    Cmd,
    RunAs,
    Env,
    Text,
    Copy,
    Download,
    EnsureDir,
    CacheDirs,
    EmptyDir,
    Remove,
    Depends,
    Container,
    Build,
    SubConfig,
    NpmConfig,
    NpmDependencies,
    YarnDependencies,
    NpmInstall,
    GemInstall,
    GemBundle,
    GemConfig,
    ComposerInstall,
    ComposerDependencies,
    ComposerConfig,
}

pub struct NameVisitor;
pub struct StepVisitor;

#[cfg(not(feature="containers"))]
pub fn builder_validator<'x>() -> V::Enum<'x> {
    // TODO(tailhook) temporarily, until we support all commands here
    V::Enum::new()
}

#[cfg(feature="containers")]
pub fn builder_validator<'x>() -> V::Enum<'x> {
    V::Enum::new()
    .option("Alpine", cmd::alpine::Alpine::config())
    .option("AlpineRepo", cmd::alpine::AlpineRepo::config())
    .option("Ubuntu", cmd::ubuntu::Ubuntu::config())
    .option("UbuntuRelease", cmd::ubuntu::UbuntuRelease::config())
    .option("UbuntuRepo", cmd::ubuntu::UbuntuRepo::config())
    .option("UbuntuPPA", cmd::ubuntu::UbuntuPPA::config())
    .option("UbuntuUniverse", cmd::ubuntu::UbuntuUniverse::config())
    .option("AptTrust", cmd::ubuntu::AptTrust::config())
    .option("Repo", cmd::packaging::Repo::config())
    .option("Install", cmd::packaging::Install::config())
    .option("BuildDeps", cmd::packaging::BuildDeps::config())
    .option("Container", cmd::subcontainer::Container::config())
    .option("SubConfig", cmd::subcontainer::SubConfig::config())
    .option("Build", cmd::subcontainer::Build::config())
    .option("Text", cmd::text::Text::config())
    .option("Copy", cmd::copy::Copy::config())

    .option("Sh", cmd::generic::Sh::config())
    .option("Cmd", cmd::generic::Cmd::config())
    .option("RunAs", cmd::generic::RunAs::config())
    .option("Remove", cmd::dirs::Remove::config())
    .option("EnsureDir", cmd::dirs::EnsureDir::config())
    .option("EmptyDir", cmd::dirs::EmptyDir::config())
    .option("CacheDirs", cmd::dirs::CacheDirs::config())
    .option("Env", cmd::generic::Env::config())
    .option("Depends", cmd::copy::Depends::config())
    .option("Git", cmd::vcs::Git::config())
    .option("GitInstall", cmd::vcs::GitInstall::config())
    .option("GitDescribe", cmd::vcs::GitDescribe::config())
    .option("Tar", cmd::tarcmd::Tar::config())
    .option("TarInstall", cmd::tarcmd::TarInstall::config())
    .option("Unzip", cmd::unzip::Unzip::config())
    .option("Download", cmd::download::Download::config())

    // Python
    .option("PipConfig", cmd::pip::PipConfig::config())
    .option("Py2Install", cmd::pip::Py2Install::config())
    .option("Py2Requirements", cmd::pip::Py2Requirements::config())
    .option("Py3Install", cmd::pip::Py3Install::config())
    .option("Py3Requirements", cmd::pip::Py3Requirements::config())

    // Node.js
    .option("NpmConfig", cmd::npm::NpmConfig::config())
    .option("NpmInstall", cmd::npm::NpmInstall::config())
    .option("NpmDependencies", cmd::npm::NpmDependencies::config())
    .option("YarnDependencies", cmd::npm::YarnDependencies::config())

    // Composer
    .option("ComposerConfig", cmd::composer::ComposerConfig::config())
    .option("ComposerInstall", cmd::composer::ComposerInstall::config())
    .option("ComposerDependencies",
        cmd::composer::ComposerDependencies::config())

    // Ruby
    .option("GemConfig", cmd::gem::GemConfig::config())
    .option("GemInstall", cmd::gem::GemInstall::config())
    .option("GemBundle", cmd::gem::GemBundle::config())
}

impl<'a> Visitor<'a> for NameVisitor {
    type Value = CommandName;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "build step is one of {}", COMMANDS.join(", "))
    }
    fn visit_str<E: de::Error>(self, val: &str) -> Result<CommandName, E> {
        use self::CommandName::*;
        let res = match val {
            "Alpine" => Alpine,
            "AlpineRepo" => AlpineRepo,
            "Ubuntu" => Ubuntu,
            "UbuntuRepo" => UbuntuRepo,
            "UbuntuRelease" => UbuntuRelease,
            "UbuntuPPA" => UbuntuPPA,
            "UbuntuUniverse" => UbuntuUniverse,
            "AptTrust" => AptTrust,
            "Repo" => Repo,
            "Install" => Install,
            "BuildDeps" => BuildDeps,
            "Git" => Git,
            "GitInstall" => GitInstall,
            "GitDescribe" => GitDescribe,
            "PipConfig" => PipConfig,
            "Py2Install" => Py2Install,
            "Py2Requirements" => Py2Requirements,
            "Py3Install" => Py3Install,
            "Py3Requirements" => Py3Requirements,
            "Tar" => Tar,
            "TarInstall" => TarInstall,
            "Unzip" => Unzip,
            "Sh" => Sh,
            "Cmd" => Cmd,
            "RunAs" => RunAs,
            "Env" => Env,
            "Text" => Text,
            "Copy" => Copy,
            "Download" => Download,
            "EnsureDir" => EnsureDir,
            "CacheDirs" => CacheDirs,
            "EmptyDir" => EmptyDir,
            "Remove" => Remove,
            "Depends" => Depends,
            "Container" => Container,
            "Build" => Build,
            "SubConfig" => SubConfig,
            "NpmConfig" => NpmConfig,
            "NpmDependencies" => NpmDependencies,
            "YarnDependencies" => YarnDependencies,
            "NpmInstall" => NpmInstall,
            "GemInstall" => GemInstall,
            "GemBundle" => GemBundle,
            "GemConfig" => GemConfig,
            "ComposerInstall" => ComposerInstall,
            "ComposerDependencies" => ComposerDependencies,
            "ComposerConfig" => ComposerConfig,
            _ => return Err(E::custom("invalid build step")),
        };
        Ok(res)
    }
}

fn decode<'x, T, V>(v: V)
    -> Result<Step, V::Error>
    where
        T: BuildStep + Deserialize<'x> + 'static,
        V: VariantAccess<'x>,
{
    v.newtype_variant::<T>().map(|x| Step(Rc::new(x) as Rc<BuildStep>))
}

impl<'a> Visitor<'a> for StepVisitor {
    type Value = Step;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "build step is one of {}", COMMANDS.join(", "))
    }
    #[cfg(not(feature="containers"))]
    fn visit_enum<A>(self, data: A) -> Result<Step, A::Error>
        where A: EnumAccess<'a>,
    {
        use self::CommandName::*;
        let (tag, v): (CommandName, _) = data.variant()?;
        match tag {
            // TODO(tailhook) temporarily, until we support all commands here
            _ => unimplemented!(),
        }
    }

    #[cfg(feature="containers")]
    fn visit_enum<A>(self, data: A) -> Result<Step, A::Error>
        where A: EnumAccess<'a>,
    {
        use self::CommandName::*;
        let (tag, v) = data.variant()?;
        match tag {
            Alpine => decode::<cmd::alpine::Alpine, _>(v),
            AlpineRepo => decode::<cmd::alpine::AlpineRepo, _>(v),
            Ubuntu => decode::<cmd::ubuntu::Ubuntu, _>(v),
            UbuntuRepo => decode::<cmd::ubuntu::UbuntuRepo, _>(v),
            UbuntuRelease => decode::<cmd::ubuntu::UbuntuRelease, _>(v),
            UbuntuPPA => decode::<cmd::ubuntu::UbuntuPPA, _>(v),
            UbuntuUniverse => decode::<cmd::ubuntu::UbuntuUniverse, _>(v),
            AptTrust => decode::<cmd::ubuntu::AptTrust, _>(v),
            Repo => decode::<cmd::packaging::Repo, _>(v),
            Install => decode::<cmd::packaging::Install, _>(v),
            BuildDeps => decode::<cmd::packaging::BuildDeps, _>(v),
            Git => decode::<cmd::vcs::Git, _>(v),
            GitInstall => decode::<cmd::vcs::GitInstall, _>(v),
            GitDescribe => decode::<cmd::vcs::GitDescribe, _>(v),
            PipConfig => decode::<cmd::pip::PipConfig, _>(v),
            Py2Install => decode::<cmd::pip::Py2Install, _>(v),
            Py2Requirements => decode::<cmd::pip::Py2Requirements, _>(v),
            Py3Install => decode::<cmd::pip::Py3Install, _>(v),
            Py3Requirements => decode::<cmd::pip::Py3Requirements, _>(v),
            Tar => decode::<cmd::tarcmd::Tar, _>(v),
            TarInstall => decode::<cmd::tarcmd::TarInstall, _>(v),
            Unzip => decode::<cmd::unzip::Unzip, _>(v),
            Sh => decode::<cmd::generic::Sh, _>(v),
            Cmd => decode::<cmd::generic::Cmd, _>(v),
            RunAs => decode::<cmd::generic::RunAs, _>(v),
            Env => decode::<cmd::generic::Env, _>(v),
            Text => decode::<cmd::text::Text, _>(v),
            Copy => decode::<cmd::copy::Copy, _>(v),
            Download => decode::<cmd::download::Download, _>(v),
            EnsureDir => decode::<cmd::dirs::EnsureDir, _>(v),
            CacheDirs => decode::<cmd::dirs::CacheDirs, _>(v),
            EmptyDir => decode::<cmd::dirs::EmptyDir, _>(v),
            Remove => decode::<cmd::dirs::Remove, _>(v),
            Depends => decode::<cmd::copy::Depends, _>(v),
            Container => decode::<cmd::subcontainer::Container, _>(v),
            Build => decode::<cmd::subcontainer::Build, _>(v),
            SubConfig => decode::<cmd::subcontainer::SubConfig, _>(v),
            NpmConfig => decode::<cmd::npm::NpmConfig, _>(v),
            NpmDependencies => decode::<cmd::npm::NpmDependencies, _>(v),
            YarnDependencies => decode::<cmd::npm::YarnDependencies, _>(v),
            NpmInstall => decode::<cmd::npm::NpmInstall, _>(v),
            GemInstall => decode::<cmd::gem::GemInstall, _>(v),
            GemBundle => decode::<cmd::gem::GemBundle, _>(v),
            GemConfig => decode::<cmd::gem::GemConfig, _>(v),
            ComposerInstall => decode::<cmd::composer::ComposerInstall, _>(v),
            ComposerDependencies
            => decode::<cmd::composer::ComposerDependencies, _>(v),
            ComposerConfig => decode::<cmd::composer::ComposerConfig, _>(v),
        }
    }
}

impl<'a> Deserialize<'a> for CommandName {
    fn deserialize<D: Deserializer<'a>>(d: D) -> Result<CommandName, D::Error>
    {
        d.deserialize_identifier(NameVisitor)
    }
}

impl<'a> Deserialize<'a> for Step {
    fn deserialize<D: Deserializer<'a>>(d: D) -> Result<Step, D::Error> {
        d.deserialize_enum("BuildStep", COMMANDS, StepVisitor)
    }
}
