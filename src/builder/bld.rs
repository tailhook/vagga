use std::fs::{File, create_dir_all, set_permissions, Permissions, remove_file};
use std::fs::{symlink_metadata};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use regex::Regex;
use scan_dir::{ScanDir};

use config::builders::Builder;
use config::builders::Builder as B;
use config::builders::Source as S;
use config::read_config;
use container::mount::{bind_mount, remount_ro};
use container::util::{clean_dir, copy_dir};
use super::super::path_util::ToRelative;
use super::commands::ubuntu;
use super::commands::alpine;
use super::commands::generic;
use super::commands::pip;
use super::commands::npm;
use super::commands::composer;
use super::commands::vcs;
use super::commands::download;
use super::tarcmd;
use version::short_version;
use builder::distrib::{DistroBox};
use builder::guard::Guard;
use builder::error::StepError;
use path_util::PathExt;
use file_util::{create_dir_mode, create_dir, shallow_copy};


pub trait BuildCommand {
    fn build(&self, ctx: &mut Guard, build: bool) -> Result<(), StepError>;
}


impl BuildCommand for Builder {
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        use builder::error::StepError as E;
        match self {
            &B::Install(ref pkgs) => {
                guard.ctx.packages.extend(pkgs.clone().into_iter());
                for i in pkgs.iter() {
                    guard.ctx.build_deps.remove(i);
                }
                if build {
                    try!(guard.distro.install(&mut guard.ctx, pkgs));
                }
            }
            &B::BuildDeps(ref pkgs) => {
                for i in pkgs.iter() {
                    if !guard.ctx.packages.contains(i) {
                        guard.ctx.build_deps.insert(i.clone());
                    }
                }
                if build {
                    try!(guard.distro.install(&mut guard.ctx, pkgs));
                }
            }
            &B::Container(ref name) => {
                let cont = guard.ctx.config.containers.get(name)
                    .expect("Subcontainer not found");  // TODO
                for b in cont.setup.iter() {
                    try!(b.build(guard, false)
                        .map_err(|e| E::SubStep(b.clone(), Box::new(e))));
                }
                if build {
                    let version = try!(short_version(&cont, &guard.ctx.config)
                        .map_err(|(s, e)| format!("step {}: {}", s, e)));
                    let path = Path::new("/vagga/base/.roots")
                        .join(format!("{}.{}", name, version)).join("root");
                    try_msg!(copy_dir(&path, &Path::new("/vagga/root"),
                                      None, None),
                        "Error copying dir {p:?}: {err}", p=path);
                }
            }
            &B::Build(ref binfo) => {
                let ref name = binfo.container;
                let cont = guard.ctx.config.containers.get(name)
                    .expect("Subcontainer not found");  // TODO
                if build {
                    let version = try!(short_version(&cont, &guard.ctx.config)
                        .map_err(|(s, e)| format!("step {}: {}", s, e)));
                    let path = Path::new("/vagga/base/.roots")
                        .join(format!("{}.{}", name, version)).join("root")
                        .join(binfo.source.rel());
                    if let Some(ref dest_rel) = binfo.path {
                        let dest = Path::new("/vagga/root")
                            .join(dest_rel.rel());
                        try_msg!(copy_dir(&path, &dest, None, None),
                            "Error copying dir {p:?}: {err}", p=path);
                    } else if let Some(ref dest_rel) = binfo.temporary_mount {
                        let dest = Path::new("/vagga/root")
                            .join(dest_rel.rel());
                        try_msg!(create_dir(&dest, false),
                            "Error creating destination dir: {err}");
                        try!(bind_mount(&path, &dest));
                        try!(remount_ro(&dest));
                        guard.ctx.mounted.push(dest);
                    }
                }
            }
            &B::SubConfig(ref sconfig) => {
                let path = match sconfig.source {
                    S::Container(ref container) => {
                        let cont = guard.ctx.config.containers.get(container)
                            .expect("Subcontainer not found");  // TODO
                        let version = try!(short_version(&cont, &guard.ctx.config)
                            .map_err(|(s, e)| format!("step {}: {}", s, e)));
                        Path::new("/vagga/base/.roots")
                            .join(format!("{}.{}", container, version))
                            .join("root").join(&sconfig.path)
                    }
                    S::Git(ref _git) => {
                        unimplemented!();
                    }
                    S::Directory => {
                        Path::new("/work").join(&sconfig.path)
                    }
                };
                let subcfg = try!(read_config(&path));
                let cont = subcfg.containers.get(&sconfig.container)
                    .expect("Subcontainer not found");  // TODO
                for b in cont.setup.iter() {
                    try!(b.build(guard, build)
                        .map_err(|e| E::SubStep(b.clone(), Box::new(e))));
                }
            }
            &B::Text(ref files) => {
                if build {
                    for (path, text) in files.iter() {
                        let realpath = Path::new("/vagga/root")
                            .join(path.rel());
                        try!(File::create(&realpath)
                            .and_then(|mut f| f.write_all(text.as_bytes()))
                            .map_err(|e| format!("Can't create file: {}", e)));
                        try!(set_permissions(&realpath,
                            Permissions::from_mode(0o644))
                            .map_err(|e| format!("Can't chmod file: {}", e)));
                    }
                }
            }
            &B::Copy(ref cinfo) => {
                if build {
                    let ref src = cinfo.source;
                    let dest = Path::new("/vagga/root").join(cinfo.path.rel());
                    let typ = try!(symlink_metadata(src)
                        .map_err(|e| E::Write(src.into(), e)));
                    if typ.is_dir() {
                        try!(create_dir_mode(&dest, typ.permissions().mode())
                            .map_err(|e| E::Write(dest.clone(), e)));
                        let re = try!(Regex::new(&cinfo.ignore_regex));
                        try!(ScanDir::all().walk(src, |iter| {
                            for (entry, _) in iter {
                                let fpath = entry.path();
                                // We know that directory is inside
                                // the source
                                let path = fpath.rel_to(src).unwrap();
                                // We know that it's decodable
                                let strpath = path.to_str().unwrap();
                                if re.is_match(strpath) {
                                    continue;
                                }
                                let fdest = dest.join(path);
                                try!(shallow_copy(&fpath, &fdest,
                                        cinfo.owner_uid, cinfo.owner_gid)
                                    .map_err(|e| E::Write(fdest, e)));
                            }
                            Ok(())
                        }).map_err(E::ScanDir).and_then(|x| x));
                    } else {
                        try!(shallow_copy(&cinfo.source, &dest,
                                          cinfo.owner_uid, cinfo.owner_gid)
                             .map_err(|e| E::Write(dest.clone(), e)));
                    }
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
                let fpath = Path::new("/vagga/root").join(path.rel());
                if fpath.is_dir() {
                    try!(clean_dir(&fpath, true));
                } else if fpath.exists() {
                    try!(remove_file(&fpath)
                        .map_err(|e| format!("Error removing file {:?}: {}",
                                             &fpath, e)));
                }
                guard.ctx.add_remove_dir(&path);
            }
            &B::EmptyDir(ref path) => {
                try!(clean_dir(path, false));
                guard.ctx.add_empty_dir(&path);
            }
            &B::EnsureDir(ref path) => {
                let fpath = Path::new("/vagga/root").join(path.rel());
                try!(create_dir_all(&fpath)
                    .map_err(|e| format!("Error creating dir: {}", e)));
                try!(set_permissions(&fpath, Permissions::from_mode(0o755))
                    .map_err(|e| format!("Error setting permissions: {}", e)));
                for mount_point in guard.ctx.container_config.volumes.keys() {
                    if path != mount_point && path.starts_with(mount_point) {
                        warn!("{0:?} directory is in the volume: {1:?}.\n\t\
                               {0:?} will be unaccessible inside the container.",
                            path,
                            mount_point);
                    }
                }
                guard.ctx.add_ensure_dir(path);
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
                try!(alpine::configure(&mut guard.distro, &mut guard.ctx,
                    version));
                if build {
                    try!(guard.distro.bootstrap(&mut guard.ctx));
                }
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
                    try!(composer::composer_install(&mut guard.distro, &mut guard.ctx,
                        pkgs));
                }
            }
            &B::ComposerRequirements(ref info) => {
                try!(composer::configure(&mut guard.ctx));
                if build {
                    try!(composer::composer_requirements(&mut guard.distro,
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
