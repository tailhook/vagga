use std::collections::HashSet;
use std::fs::{read_dir, read_link, remove_file};
use std::io::{stdout, stderr};
use std::path::Path;
use std::str::FromStr;

use argparse::{ArgumentParser, PushConst, StoreTrue};

use super::setup;
use super::Wrapper;
use container::util::clean_dir;
use file_util::read_visible_entries;
use path_util::PathExt;
use wrapper::build::get_version_hash;


#[derive(Clone, Copy)]
enum Action {
    Temporary,
    Old,
    Unused,
    Everything,
    Transient,
}


pub fn clean_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut global = false;
    let mut dry_run = false;
    let mut actions = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Performs various cleanup tasks
            ");
        ap.refer(&mut actions)
          .add_option(&["--tmp", "--tmp-folders"],
                PushConst(Action::Temporary),
                "Clean temporary containers (failed builds)")
          .add_option(&["--old", "--old-containers"], PushConst(Action::Old), "
                Clean old versions of containers (those which doesn't have a \
                symlink in .vagga)")
          .add_option(&["--unused"], PushConst(Action::Unused), "
                Clean unused containers, or versions thereof. (This is not \
                `--old` for historical reasons, we will probably merge the \
                commands later on)")
          .add_option(&["--transient"], PushConst(Action::Transient),
                "Clean unneeded transient folders (left from containers with
                 `write-mode` set to transient-something). The pid of process
                 is checked for liveness first.")
          .add_option(&["--everything"], PushConst(Action::Everything),
                "Clean whole `.vagga` folder. Useful when deleting a project.
                 With ``--global`` cleans whole storage-dir and cache-dir")
          .required();
        ap.refer(&mut global)
          .add_option(&["--global"], StoreTrue,
                "Apply cleanup command to all containers. Works only \
                if `storage-dir` is configured in settings");
        ap.refer(&mut dry_run)
          .add_option(&["-n", "--dry-run"], StoreTrue,
                "Dry run. Don't delete everything, just print");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(1);
            }
        }
    }
    if global && wrapper.ext_settings.storage_dir.is_none() {
        error!("The --global flag is only meaningful if you configure \
            storage-dir in settings");
        return Ok(2);
    }
    for action in actions.iter() {
        let res = match *action {
            Action::Temporary => clean_temporary(wrapper, global, dry_run),
            Action::Old => clean_old(wrapper, global, dry_run),
            Action::Unused => clean_unused(wrapper, global, dry_run),
            Action::Transient => clean_transient(wrapper, global, dry_run),
            _ => unimplemented!(),
        };
        match res {
            Ok(()) => {}
            Err(err) => {
                error!("Error cleaning up: {}", err);
                return Ok(3);
            }
        }
    }
    return Ok(0);
}

fn clean_dir_wrapper(path: &Path, dry_run: bool) -> Result<(), String> {
    // TODO(tailhook) chroot to dir for removing
    if dry_run {
        println!("Would remove {:?}", path);
    } else {
        debug!("Removing {:?}", path);
        try!(clean_dir(path, true));
    }
    Ok(())
}

fn clean_temporary(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match try!(setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings))
    {
        Some(base) => base,
        None => {
            warn!("No vagga directory exists");
            return Ok(());
        }
    };
    let roots = base.join(".roots");
    for entry in try_msg!(read_dir(&roots),
        "Can't read dir {r:?}: {err}", r=roots)
    {
        let entry = try_msg!(entry, "Can't read dir {r:?}: {err}", r=roots);
        let typ = try_msg!(entry.file_type(),
            "Can't stat {p:?}: {err}", p=entry.path());
        if typ.is_dir() &&
           entry.file_name()[..].to_str().map(|n| n.starts_with(".tmp"))
                                         .unwrap_or(false)
        {
            try!(clean_dir_wrapper(&entry.path(), dry_run));
        }
    }

    return Ok(());
}

fn clean_old(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match try!(setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings))
    {
        Some(base) => base,
        None => {
            warn!("No vagga directory exists");
            return Ok(());
        }
    };
    let useful: HashSet<String> = try_msg!(
        read_visible_entries(&wrapper.project_root.join(".vagga")),
            "Can't read vagga directory: {err}")
        .into_iter()
        .filter_map(|path| read_link(&path)
             .map_err(|e| warn!("Can't readlink {:?}: {}", path, e))
             .ok()
             .and_then(|f| {
                 // The container name is next to the last component
                 f.iter().rev().nth(1)
                 .and_then(|x| x.to_str()).map(ToString::to_string)
             }))
        .collect();
    debug!("Useful images {:?}", useful);

    let roots = base.join(".roots");
    for entry in try_msg!(read_dir(&roots),
                         "Can't read dir {dir:?}: {err}", dir=roots)
    {
        let entry = try_msg!(entry,
                             "Can't read dir {dir:?}: {err}", dir=roots);
        let typ = try_msg!(entry.file_type(),
            "Can't stat {p:?}: {err}", p=entry.path());
        if !typ.is_dir() {
            try_msg!(remove_file(&entry.path()),
                "Can't remove file {p:?}: {err}", p=entry.path());
        } else if !typ.is_dir() || entry.file_name()[..].to_str()
            .map(|n| !useful.contains(&n.to_string()))
            .unwrap_or(false)
        {
            try!(clean_dir_wrapper(&entry.path(), dry_run));
        }
    }

    return Ok(());
}

fn clean_transient(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match try!(setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings))
    {
        Some(base) => base,
        None => {
            warn!("No vagga directory exists");
            return Ok(());
        }
    };
    let procfs = Path::new("/proc");
    for entry in try_msg!(read_dir(&base.join(".transient")),
                        "Can't read .vagga/.transient dir: {err}")
    {
        let entry = try_msg!(entry, "Error reading .vagga/transient: {err}");
        if let Some(fname) = entry.file_name()[..].to_str() {
            if let Some(idx) = fname.find('.') {
                if u32::from_str(&fname[idx+1..]).is_ok() &&
                    procfs.join(&fname[idx+1..]).exists()
                {
                    continue;
                }
            }
        }
        try!(clean_dir_wrapper(&entry.path(), dry_run));
    }

    return Ok(());
}

fn clean_unused(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }

    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    let mut useful: HashSet<String> = HashSet::new();
    for (name, _) in &wrapper.config.containers {
        if let Some(version) = try!(get_version_hash(name, wrapper)) {
            useful.insert(format!("{}.{}", name, &version[..8]));
        }
    }
    debug!("Useful images {:?}", useful);

    let roots = Path::new("/vagga/base/.roots");
    for entry in try_msg!(read_dir(&roots),
                         "Can't read dir {dir:?}: {err}", dir=roots)
    {
        let entry = try_msg!(entry,
                             "Can't read dir {dir:?}: {err}", dir=roots);
        let typ = try_msg!(entry.file_type(),
            "Can't stat {p:?}: {err}", p=entry.path());
        if !typ.is_dir() {
            try_msg!(remove_file(&entry.path()),
                "Can't remove file {p:?}: {err}", p=entry.path());
        } else if !typ.is_dir() || entry.file_name()[..].to_str()
            .map(|n| !useful.contains(&n.to_string()))
            .unwrap_or(false)
        {
            try!(clean_dir_wrapper(&entry.path(), dry_run));
        }
    }

    return Ok(());
}
