use std::fs::{File, create_dir_all, set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use config::builders::Builder;
use config::builders::Builder as B;
use config::builders::Source as S;
use config::read_config;
use container::util::{clean_dir, copy_dir};
use container::vagga::container_ver;

use super::context::BuildContext;
use super::commands::debian;
use super::commands::alpine;
use super::commands::generic;
use super::commands::pip;
use super::commands::npm;
use super::commands::vcs;
use super::tarcmd;
use super::context::Distribution as Distr;


pub trait BuildCommand {
    fn build(&self, ctx: &mut BuildContext, build: bool) -> Result<(), String>;
}

fn configure_ubuntu(ctx: &mut BuildContext, name: &String)
    -> Result<(), String>
{
    if let Distr::Unknown = ctx.distribution {
        ctx.distribution = Distr::Ubuntu(debian::UbuntuInfo {
            release: name.to_string(),
            apt_update: true,
            has_universe: false,
        });
    } else {
        return Err(format!("Conflicting distribution"));
    };
    try!(ctx.add_cache_dir(Path::new("/var/cache/apt"),
                           "apt-cache".to_string()));
    try!(ctx.add_cache_dir(Path::new("/var/lib/apt/lists"),
                          "apt-lists".to_string()));
    ctx.environ.insert("DEBIAN_FRONTEND".to_string(),
                       "noninteractive".to_string());
    ctx.environ.insert("LANG".to_string(),
                       "en_US.UTF-8".to_string());
    ctx.environ.insert("PATH".to_string(),
                       "/usr/local/sbin:/usr/local/bin:\
                        /usr/sbin:/usr/bin:/sbin:/bin:\
                        /usr/games:/usr/local/games\
                        ".to_string());
    Ok(())
}

impl BuildCommand for Builder {
    fn build(&self, ctx: &mut BuildContext, build: bool)
        -> Result<(), String>
    {
        match self {
            &B::Install(ref pkgs) => {
                ctx.packages.extend(pkgs.clone().into_iter());
                for i in pkgs.iter() {
                    ctx.build_deps.remove(i);
                }
                if build {
                    match ctx.distribution {
                        Distr::Unknown => {
                            return Err(format!("Unknown distribution"));
                        }
                        Distr::Ubuntu(_) => {
                            try!(debian::apt_install(ctx, pkgs));
                        }
                        Distr::Alpine(_) => {
                            try!(alpine::install(ctx, pkgs));
                        }
                    }
                }
            }
            &B::BuildDeps(ref pkgs) => {
                for i in pkgs.iter() {
                    if !ctx.packages.contains(i) {
                        ctx.build_deps.insert(i.clone());
                    }
                }
                if build {
                    match ctx.distribution {
                        Distr::Unknown => {
                            return Err(format!("Unknown distribution"));
                        }
                        Distr::Ubuntu(_) => {
                            try!(debian::apt_install(ctx, pkgs));
                        }
                        Distr::Alpine(_) => {
                            try!(alpine::install(ctx, pkgs));
                        }
                    }
                }
            }
            &B::Container(ref name) => {
                let cont = ctx.config.containers.get(name)
                    .expect("Subcontainer not found");  // TODO
                for b in cont.setup.iter() {
                    try!(b.build(ctx, false));
                }
                if build {
                    let version = try!(container_ver(name));
                    let path = Path::new("/vagga/base/.roots")
                        .join(version).join("root");
                    try!(copy_dir(&path, &Path::new("/vagga/root")));
                }
            }
            &B::SubConfig(ref sconfig) => {
                let path = match sconfig.source {
                    S::Container(ref container) => {
                        let version = try!(container_ver(container));
                        Path::new("/vagga/base/.roots")
                            .join(version).join("root")
                            .join(&sconfig.path)
                    }
                    S::Git(ref git) => {
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
                    try!(b.build(ctx, build));
                }
            }
            &B::Text(ref files) => {
                if build {
                    for (path, text) in files.iter() {
                        let realpath = Path::new("/vagga/root").join(
                            path.path_relative_from(&Path::new("/")).unwrap());
                        try!(File::create(&realpath)
                            .and_then(|mut f| f.write_str(text))
                            .map_err(|e| format!("Can't create file: {}", e)));
                        try!(set_permissions(&realpath,
                            Permissions::from_mode(0o755))
                            .map_err(|e| format!("Can't chmod file: {}", e)));
                    }
                }
            }
            &B::Ubuntu(ref name) => {
                try!(configure_ubuntu(ctx, name));

                if build {
                    try!(debian::fetch_ubuntu_core(ctx, name, debian::BuildType::Daily));
                    try!(debian::init_ubuntu_core(ctx));
                }
            }
            &B::UbuntuRelease(ref release_info) => {
                if build {
                    try!(debian::fetch_ubuntu_core(ctx, &release_info.version, debian::BuildType::Release));
                }

                let codename = try!(debian::read_ubuntu_codename());
                try!(configure_ubuntu(ctx, &codename));

                if build {
                    try!(debian::init_ubuntu_core(ctx));
                }
            }
            &B::UbuntuRepo(ref repo) => {
                if build {
                    try!(debian::add_debian_repo(ctx, repo));
                }
            }
            &B::UbuntuUniverse => {
                match ctx.distribution {
                    Distr::Ubuntu(ref mut ubuntu) => {
                        ubuntu.has_universe = true;
                    }
                    _ => unreachable!(),
                }
                if build {
                    try!(debian::ubuntu_add_universe(ctx));
                }
            }
            &B::Sh(ref text) => {
                if build {
                    try!(generic::run_command(ctx,
                        &["/bin/sh".to_string(),
                          "-c".to_string(),
                          text.to_string()]));
                }
            }
            &B::Cmd(ref cmd) => {
                if build {
                    try!(generic::run_command(ctx, &cmd));
                }
            }
            &B::Env(ref pairs) => {
                for (k, v) in pairs.iter() {
                    ctx.environ.insert(k.clone(), v.clone());
                }
            }
            &B::Remove(ref path) => {
                try!(clean_dir(path, true));
                ctx.add_remove_dir(path.clone());
            }
            &B::EmptyDir(ref path) => {
                try!(clean_dir(path, false));
                ctx.add_empty_dir(path.clone());
            }
            &B::EnsureDir(ref path) => {
                let fpath = path.path_relative_from(&Path::new("/")).unwrap();
                let fpath = Path::new("/vagga/root").join(fpath);
                try!(create_dir_all(&fpath)
                    .map_err(|e| format!("Error creating dir: {}", e)));
                try!(set_permissions(&fpath, Permissions::from_mode(0o755))
                    .map_err(|e| format!("Error setting permissions: {}", e)));
                ctx.add_ensure_dir(path.clone());
            }
            &B::CacheDirs(ref pairs) => {
                for (k, v) in pairs.iter() {
                    try!(ctx.add_cache_dir(k.clone(), v.clone()));
                }
            }
            &B::Depends(_) => {
            }
            &B::Git(ref git) => {
                if build {
                    try!(vcs::git_command(ctx, git));
                }
            }
            &B::GitInstall(ref git) => {
                if build {
                    try!(vcs::git_install(ctx, git));
                }
            }
            &B::Tar(ref tar) => {
                if build {
                    try!(tarcmd::tar_command(ctx, tar));
                }
            }
            &B::TarInstall(ref tar_inst) => {
                if build {
                    try!(tarcmd::tar_install(ctx, tar_inst));
                }
            }
            &B::Alpine(ref version) => {
                if let Distr::Unknown = ctx.distribution {
                    ctx.distribution = Distr::Alpine(alpine::AlpineInfo {
                        version: version.to_string(),
                        base_setup: false,
                    });
                } else {
                    return Err(format!("Conflicting distribution"));
                };
                try!(ctx.add_cache_dir(Path::new("/etc/apk/cache"),
                                       "alpine-cache".to_string()));
                ctx.environ.insert("LANG".to_string(),
                                   "en_US.UTF-8".to_string());
                ctx.environ.insert("PATH".to_string(),
                                   "/usr/local/sbin:/usr/local/bin:\
                                    /usr/sbin:/usr/bin:/sbin:/bin\
                                    ".to_string());
                if build {
                    try!(alpine::setup_base(ctx, version));
                }
            }
            &B::PipConfig(ref pip_settings) => {
                ctx.pip_settings = pip_settings.clone();
            }
            &B::Py2Install(ref pkgs) => {
                try!(pip::configure(ctx));
                if build {
                    try!(pip::pip_install(ctx, 2, pkgs));
                }
            }
            &B::Py3Install(ref pkgs) => {
                try!(pip::configure(ctx));
                if build {
                    try!(pip::pip_install(ctx, 3, pkgs));
                }
            }
            &B::Py2Requirements(ref fname) => {
                try!(pip::configure(ctx));
                if build {
                    try!(pip::pip_requirements(ctx, 2, fname));
                }
            }
            &B::Py3Requirements(ref fname) => {
                try!(pip::configure(ctx));
                if build {
                    try!(pip::pip_requirements(ctx, 3, fname));
                }
            }
            &B::NpmInstall(ref pkgs) => {
                if let Distr::Unknown = ctx.distribution {
                    B::Alpine(alpine::LATEST_VERSION.to_string())
                        .build(ctx, build);
                }
                if build {
                    try!(npm::npm_install(ctx, pkgs));
                }
            }
        }
        if build {
            try!(ctx.timelog.mark(format_args!("Step: {:?}", self))
                .map_err(|e| format!("Can't write timelog: {}", e)));
        }
        Ok(())
    }
}
