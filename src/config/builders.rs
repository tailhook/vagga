use std::fmt;
use std::rc::Rc;

use quire::validate as V;
use serde::de::{self, Deserializer, Deserialize, EnumAccess, VariantAccess, Visitor};
use serde::ser::{Serializer, Serialize};

use crate::build_step::{Step, BuildStep};
use crate::builder::commands as cmd;

macro_rules! define_commands {
    ($($module: ident :: $item: ident,)*) => {
        const COMMANDS: &'static [&'static str] = &[
            $(stringify!($item),)*
        ];
        pub enum CommandName {
            $($item,)*
        }
        pub fn builder_validator<'x>() -> V::Enum<'x> {
            V::Enum::new()
            $(
                .option(stringify!($item), cmd::$module::$item::config())
            )*
        }
        impl<'a> Visitor<'a> for NameVisitor {
            type Value = CommandName;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "build step is one of {}", COMMANDS.join(", "))
            }
            fn visit_str<E: de::Error>(self, val: &str)
                -> Result<CommandName, E>
            {
                use self::CommandName::*;
                let res = match val {
                    $(
                        stringify!($item) => $item,
                    )*
                    _ => return Err(E::custom("invalid build step")),
                };
                Ok(res)
            }
        }
        impl<'a> Visitor<'a> for StepVisitor {
            type Value = Step;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "build step is one of {}", COMMANDS.join(", "))
            }
            fn visit_enum<A>(self, data: A) -> Result<Step, A::Error>
                where A: EnumAccess<'a>,
            {
                use self::CommandName::*;
                let (tag, v) = data.variant()?;
                match tag {
                    $(
                        $item => decode::<cmd::$module::$item, _>(v),
                    )*
                }
            }
        }
        impl Serialize for Step {
            fn serialize<S: Serializer>(&self, s: S)
                -> Result<S::Ok, S::Error>
            {

                if false { unreachable!() }
                $(
                    else if let Some(b) =
                            self.0.downcast_ref::<cmd::$module::$item>()
                    {
                        b.serialize(s)
                    }
                )*
                else {
                    unreachable!("all steps should be serializeable");
                }
            }
        }
    }
}

define_commands! {
    alpine::Alpine,
    alpine::AlpineRepo,
    ubuntu::Ubuntu,
    ubuntu::UbuntuRepo,
    ubuntu::UbuntuRelease,
    ubuntu::UbuntuPPA,
    ubuntu::UbuntuUniverse,
    ubuntu::AptTrust,
    packaging::Repo,
    packaging::Install,
    packaging::BuildDeps,
    vcs::Git,
    vcs::GitInstall,
    vcs::GitDescribe,
    pip::PipConfig,
    pip::Py2Install,
    pip::Py2Requirements,
    pip::Py3Install,
    pip::Py3Requirements,
    tarcmd::Tar,
    tarcmd::TarInstall,
    unzip::Unzip,
    generic::Sh,
    generic::Cmd,
    generic::RunAs,
    generic::Env,
    text::Text,
    copy::Copy,
    download::Download,
    dirs::EnsureDir,
    dirs::CacheDirs,
    dirs::EmptyDir,
    dirs::Remove,
    copy::Depends,
    subcontainer::Container,
    subcontainer::Build,
    subcontainer::SubConfig,
    npm::NpmConfig,
    npm::NpmDependencies,
    npm::YarnDependencies,
    npm::NpmInstall,
    gem::GemInstall,
    gem::GemBundle,
    gem::GemConfig,
    composer::ComposerInstall,
    composer::ComposerDependencies,
    composer::ComposerConfig,
}

pub struct NameVisitor;
pub struct StepVisitor;


fn decode<'x, T, V>(v: V)
    -> Result<Step, V::Error>
    where
        T: BuildStep + Deserialize<'x> + 'static,
        V: VariantAccess<'x>,
{
    v.newtype_variant::<T>().map(|x| Step(Rc::new(x) as Rc<dyn BuildStep>))
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
