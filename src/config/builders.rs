use std::rc::Rc;
use std::path::PathBuf;

use quire::validate as V;
use libc::{uid_t, gid_t};
use rustc_serialize::{Decodable, Decoder};
use builder::commands as cmd;

use build_step::{Step, BuildStep};

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct PacmanRepo {
    pub name: String,
    pub url: String,
}

#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct FileInfo {
    pub name: PathBuf,
    pub contents: String,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Debug)]
pub struct Copy {
    pub source: PathBuf,
    pub path: PathBuf,
    pub owner_uid: Option<uid_t>,
    pub owner_gid: Option<gid_t>,
    pub ignore_regex: String,
}


pub fn builder_validator<'x>() -> V::Enum<'x> {
    V::Enum::new()
    .option("Alpine", cmd::alpine::Alpine::config())
    .option("Ubuntu", cmd::ubuntu::Ubuntu::config())
    .option("UbuntuRelease", cmd::ubuntu::UbuntuRelease::config())
    .option("UbuntuRepo", cmd::ubuntu::UbuntuRepo::config())
    .option("UbuntuPPA", cmd::ubuntu::UbuntuPPA::config())
    .option("UbuntuUniverse", cmd::ubuntu::UbuntuUniverse::config())
    .option("AptTrust", cmd::ubuntu::AptTrust::config())
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
    .option("Depends", cmd::generic::Depends::config())
    .option("Git", cmd::vcs::Git::config())
    .option("GitInstall", cmd::vcs::GitInstall::config())
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
        "Git" => step(cmd::vcs::Git::decode(d)),
        "GitInstall" => step(cmd::vcs::GitInstall::decode(d)),
        "PipConfig" => step(cmd::pip::PipConfig::decode(d)),
        "Py2Install" => step(cmd::pip::Py2Install::decode(d)),
        "Py2Requirements" => step(cmd::pip::Py2Requirements::decode(d)),
        "Py3Install" => step(cmd::pip::Py3Install::decode(d)),
        "Py3Requirements" => step(cmd::pip::Py3Requirements::decode(d)),
        "Tar" => step(cmd::tarcmd::Tar::decode(d)),
        "TarInstall" => step(cmd::tarcmd::TarInstall::decode(d)),
        "Unzip" => step(cmd::unzip::Unzip::decode(d)),
        "Sh" => step(cmd::generic::Sh::decode(d)),
        "Cmd" => step(cmd::generic::Cmd::decode(d)),
        "RunAs" => step(cmd::generic::RunAs::decode(d)),
        "Env" => step(cmd::generic::Env::decode(d)),
        "Text" => step(cmd::text::Text::decode(d)),
        "Copy" => step(cmd::copy::Copy::decode(d)),
        "Download" => step(cmd::download::Download::decode(d)),
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
        "GemInstall" => step(cmd::gem::GemInstall::decode(d)),
        "GemBundle" => step(cmd::gem::GemBundle::decode(d)),
        "GemConfig" => step(cmd::gem::GemConfig::decode(d)),
        "ComposerInstall" => step(cmd::composer::ComposerInstall::decode(d)),
        "ComposerDependencies"
        => step(cmd::composer::ComposerDependencies::decode(d)),
        "ComposerConfig" => step(cmd::composer::ComposerConfig::decode(d)),
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
