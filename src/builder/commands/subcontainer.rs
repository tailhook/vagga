use std::path::Path;

use config::read_config;
use config::{Container, Config};
use config::builders::{BuildInfo, SubConfigInfo};
use builder::guard::Guard;
use builder::error::StepError;
use version::short_version;
use container::mount::{bind_mount, remount_ro};
use container::util::{copy_dir};
use file_util::{create_dir};
use path_util::ToRelative;
use builder::bld::BuildCommand;

use builder::error::StepError as E;
use config::builders::Source as S;


pub fn build(binfo: &BuildInfo, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
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
    Ok(())
}

fn real_build(name: &String, cont: &Container, guard: &mut Guard)
    -> Result<(), StepError>
{
    let version = try!(short_version(&cont, &guard.ctx.config)
        .map_err(|(s, e)| format!("step {}: {}", s, e)));
    let path = Path::new("/vagga/base/.roots")
        .join(format!("{}.{}", name, version)).join("root");
    try_msg!(copy_dir(&path, &Path::new("/vagga/root"),
                      None, None),
        "Error copying dir {p:?}: {err}", p=path);
    Ok(())
}

pub fn clone(name: &String, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let cont = guard.ctx.config.containers.get(name)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        try!(b.build(guard, false)
            .map_err(|e| E::SubStep(Box::new(b.clone()), Box::new(e))));
    }
    if build {
        try!(real_build(name, cont, guard));
    }
    Ok(())
}

fn find_config(cfg: &SubConfigInfo, guard: &mut Guard)
    -> Result<Config, StepError>
{
    let path = match cfg.source {
        S::Container(ref container) => {
            let cont = guard.ctx.config.containers.get(container)
                .expect("Subcontainer not found");  // TODO
            let version = try!(short_version(&cont, &guard.ctx.config)
                .map_err(|(s, e)| format!("step {}: {}", s, e)));
            Path::new("/vagga/base/.roots")
                .join(format!("{}.{}", container, version))
                .join("root").join(&cfg.path)
        }
        S::Git(ref _git) => {
            unimplemented!();
        }
        S::Directory => {
            Path::new("/work").join(&cfg.path)
        }
    };
    Ok(try!(read_config(&path)))
}

pub fn subconfig(cfg: &SubConfigInfo, guard: &mut Guard, build: bool)
    -> Result<(), StepError>
{
    let subcfg = try!(find_config(cfg, guard));
    let cont = subcfg.containers.get(&cfg.container)
        .expect("Subcontainer not found");  // TODO
    for b in cont.setup.iter() {
        try!(b.build(guard, build)
            .map_err(|e| E::SubStep(Box::new(b.clone()), Box::new(e))));
    }
    Ok(())
}
