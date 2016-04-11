use config::Step;
use config::builders::Source as S;
use container::util::{clean_dir};
use super::commands::ubuntu;
use super::commands::alpine;
use super::commands::generic;
use super::commands::pip;
use super::commands::gem;
use super::commands::npm;
use super::commands::composer;
use super::commands::vcs;
use super::commands::download;
use super::commands::subcontainer;
use super::commands::copy;
use super::commands::text;
use super::commands::dirs;
use super::commands::packaging;
use super::tarcmd;
use builder::distrib::{DistroBox};
use builder::guard::Guard;
use builder::error::StepError;

use build_step::BuildStep;


pub trait BuildCommand {
    fn build(&self, ctx: &mut Guard, build: bool) -> Result<(), StepError>;
}

impl BuildCommand for Step {
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        //self.0.build(guard, build)
        unimplemented!();
    }
}

/*
impl BuildCommand for Builder {
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        match self {
            &B::Install(ref pkgs) => {
                try!(packaging::install(pkgs, guard, build))
            }
            &B::BuildDeps(ref pkgs) => {
                try!(packaging::build_deps(pkgs, guard, build))
            }
            &B::Container(ref name) => {
                try!(subcontainer::clone(name, guard, build))
            }
            &B::Build(ref binfo) => {
                try!(subcontainer::build(binfo, guard, build))
            }
            &B::SubConfig(ref sconfig) => {
                try!(subcontainer::subconfig(sconfig, guard, build))
            }
            &B::Text(ref files) => {
                if build {
                    try!(text::write_text_files(files, guard))
                }
            }
            &B::Copy(ref cinfo) => {
                if build {
                    try!(copy::copy(cinfo, guard))
                }
            }
            &B::Ubuntu(ref codename) => {
                try!(ubuntu::configure_simple(guard, codename));
                if build {
                    try!(guard.distro.bootstrap(&mut guard.ctx));
                }
            }
            &B::UbuntuRelease(ref release_info) => {
                try!(ubuntu::configure(guard, release_info));
                if build {
                    try!(guard.distro.bootstrap(&mut guard.ctx));
                }
            }
            &B::UbuntuRepo(ref repo) => {
                if build {
                    let ref mut ctx = guard.ctx;
                    try!(guard.distro.specific(|u: &mut ubuntu::Ubuntu| {
                        try!(u.add_debian_repo(ctx, repo));
                        Ok(())
                    }));
                }
            }
            &B::UbuntuPPA(ref name) => {
                if build {
                    let ref mut ctx = guard.ctx;
                    try!(guard.distro.specific(|u: &mut ubuntu::Ubuntu| {
                        u.add_ubuntu_ppa(ctx, name)
                    }));
                }
            }
            &B::AptTrust(ref key) => {
                if build {
                    let ref mut ctx = guard.ctx;
                    try!(guard.distro.specific(|u: &mut ubuntu::Ubuntu| {
                        u.add_apt_key(ctx, key)
                    }));
                }
            }
            &B::UbuntuUniverse => {
                let ref mut ctx = guard.ctx;
                try!(guard.distro.specific(|u: &mut ubuntu::Ubuntu| {
                    try!(u.enable_universe());
                    if build {
                        try!(u.add_universe(ctx));
                    }
                    Ok(())
                }));
            }
            &B::Sh(ref text) => {
                if build {
                    try!(generic::run_command(&mut guard.ctx,
                        &["/bin/sh".to_string(),
                          "-exc".to_string(),
                          text.to_string()]));
                }
            }
            &B::Cmd(ref cmd) => {
                if build {
                    try!(generic::run_command(&mut guard.ctx, &cmd));
                }
            }
            &B::Env(ref pairs) => {
                for (k, v) in pairs.iter() {
                    guard.ctx.environ.insert(k.clone(), v.clone());
                }
            }
            &B::Remove(ref path) => {
                try!(dirs::remove(path, guard))
            }
            &B::EmptyDir(ref path) => {
                try!(clean_dir(path, false));
                guard.ctx.add_empty_dir(&path);
            }
            &B::EnsureDir(ref path) => {
                try!(dirs::ensure(path, guard));
            }
            &B::CacheDirs(ref pairs) => {
                for (k, v) in pairs.iter() {
                    try!(guard.ctx.add_cache_dir(k, v.clone()));
                }
            }
            &B::Depends(_) => {
            }
            &B::Git(ref git) => {
                if build {
                    try!(vcs::git_command(&mut guard.ctx, git));
                }
            }
            &B::GitInstall(ref git) => {
                if build {
                    try!(vcs::git_install(&mut guard.ctx, git));
                }
            }
            &B::Tar(ref tar) => {
                if build {
                    try!(tarcmd::tar_command(&mut guard.ctx, tar));
                }
            }
            &B::TarInstall(ref tar_inst) => {
                if build {
                    try!(tarcmd::tar_install(&mut guard.ctx, tar_inst));
                }
            }
            &B::Download(ref dlinfo) => {
                if build {
                    try!(download::download(&mut guard.ctx, dlinfo));
                }
            }
            &B::Alpine(ref version) => {
                try!()
            }
            &B::PipConfig(ref pip_settings) => {
                guard.ctx.pip_settings = pip_settings.clone();
            }
            &B::Py2Install(ref pkgs) => {
                try!(pip::configure(&mut guard.ctx));
                if build {
                    try!(pip::pip_install(&mut guard.distro, &mut guard.ctx,
                        2, pkgs));
                }
            }
            &B::Py3Install(ref pkgs) => {
                try!(pip::configure(&mut guard.ctx));
                if build {
                    try!(pip::pip_install(&mut guard.distro, &mut guard.ctx,
                        3, pkgs));
                }
            }
            &B::Py2Requirements(ref fname) => {
                try!(pip::configure(&mut guard.ctx));
                if build {
                    try!(pip::pip_requirements(&mut guard.distro,
                        &mut guard.ctx, 2, fname));
                }
            }
            &B::Py3Requirements(ref fname) => {
                try!(pip::configure(&mut guard.ctx));
                if build {
                    try!(pip::pip_requirements(&mut guard.distro,
                        &mut guard.ctx, 3, fname));
                }
            }
            &B::PyFreeze(_) => unimplemented!(),
            &B::GemConfig(ref gem_settings) => {
                guard.ctx.gem_settings = gem_settings.clone();
            }
            &B::GemInstall(ref pkgs) => {
                if build {
                    try!(gem::install(&mut guard.distro, &mut guard.ctx, pkgs));
                }
            }
            &B::GemBundle(ref info) => {
                if build {
                    try!(gem::bundle(&mut guard.distro, &mut guard.ctx, info));
                }
            }
            &B::NpmConfig(ref npm_settings) => {
                guard.ctx.npm_settings = npm_settings.clone();
            }
            &B::NpmInstall(ref pkgs) => {
                try!(guard.distro.npm_configure(&mut guard.ctx));
                if build {
                    try!(npm::npm_install(&mut guard.distro, &mut guard.ctx,
                        pkgs));
                }
            }
            &B::NpmDependencies(ref info) => {
                try!(guard.distro.npm_configure(&mut guard.ctx));
                if build {
                    try!(npm::npm_deps(&mut guard.distro, &mut guard.ctx,
                        info));
                }
            }
            &B::ComposerConfig(ref composer_settings) => {
                guard.ctx.composer_settings = composer_settings.clone();
            }
            &B::ComposerInstall(ref pkgs) => {
                try!(composer::configure(&mut guard.ctx));
                if build {
                    try!(composer::composer_install(&mut guard.distro,
                        &mut guard.ctx, pkgs));
                }
            }
            &B::ComposerDependencies(ref info) => {
                try!(composer::configure(&mut guard.ctx));
                if build {
                    try!(composer::composer_dependencies(&mut guard.distro,
                        &mut guard.ctx, info));
                }
            }
        }
        if build {
            try!(guard.ctx.timelog.mark(format_args!("Step: {:?}", self))
                .map_err(|e| format!("Can't write timelog: {}", e)));
        }
        Ok(())
    }
}
*/

/*
impl BuildStep for Builder {
    fn is_dependent_on(&self) -> Option<&str> {
        match self {
            &B::Container(ref name) => {
                Some(name)
            }
            &B::Build(ref binfo) => {
                Some(&binfo.container)
            }
            &B::SubConfig(ref cfg) => {
                match cfg.source {
                    S::Directory => None,
                    S::Container(ref name) => Some(name),
                    S::Git(ref _git) => None,
                }
            }
            _ => None,
        }
    }
}
*/
