use std::collections::HashSet;
use std::fs::{read_dir, read_link, remove_file, metadata};
use std::io::{self, stdout, stderr};
use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::os::unix::fs::MetadataExt;

use argparse::{ArgumentParser, PushConst, StoreTrue, StoreOption};
use scan_dir::ScanDir;
use humantime;

use crate::config::command::MainCommand::{Supervise, Command, CapsuleCommand};
use crate::config::volumes::Volume::Persistent;
use crate::container::util::clean_dir;
use crate::file_util::{read_visible_entries, Lock};
use crate::wrapper::build::get_version_hash;

use super::setup;
use super::Wrapper;


#[derive(Clone, Copy)]
enum Action {
    Temporary,
    Old,
    Unused,
    Everything,
    Transient,
    Volumes,
    UnusedVolumes,
}


pub fn clean_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut global = false;
    let mut dry_run = false;
    let mut actions = vec!();
    let mut duration = None::<humantime::Duration>;
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
          .add_option(&["--unused-volumes"], PushConst(Action::UnusedVolumes),
                "Remove `!Persistent` volumes that are not used by any \
                 command or container of the current config")
          .add_option(&["--volumes"], PushConst(Action::Volumes),
                "Remove all `!Persistent` volumes. So they are reinitialized \
                 on the next start of the command")
          .required();
        ap.refer(&mut global)
          .add_option(&["--global"], StoreTrue,
                "Apply cleanup command to all the projects
                in the `storage-dir`. Works only \
                if `storage-dir` is configured in settings");
        ap.refer(&mut dry_run)
          .add_option(&["-n", "--dry-run"], StoreTrue,
                "Dry run. Don't delete everything, just print");
        ap.refer(&mut duration)
          .add_option(&["--at-least"], StoreOption, "
            Only in combination with `--unused`. Treat as unused \
            containers that are unused for specified time, rather than \
            the ones not used by current version of config");
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
    let duration = duration.map(|x| x.into());
    for action in actions.iter() {
        let res = match *action {
            Action::Temporary => clean_temporary(wrapper, global, dry_run),
            Action::Old => clean_old(wrapper, global, dry_run),
            Action::Unused => {
                if global {
                    if let Some(duration) = duration {
                        global_clean_unused(wrapper, duration, dry_run)
                    } else {
                        panic!("no global cleanup without --at-least");
                    }
                } else {
                    clean_unused(wrapper, duration, dry_run)
                }
            }
            Action::Transient => clean_transient(wrapper, global, dry_run),
            Action::Everything => clean_everything(wrapper, global, dry_run),
            Action::UnusedVolumes => {
                clean_volumes(wrapper, global, dry_run, false)
            }
            Action::Volumes => {
                clean_volumes(wrapper, global, dry_run, true)
            }
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

fn clean_dir_wrapper(path: &Path,
        remove_dir_itself: bool, dry_run: bool) -> Result<(), String> {
    // TODO(tailhook) chroot to dir for removing
    if dry_run {
        println!("Would remove {:?}", path);
    } else {
        let mut n = path.to_path_buf().into_os_string();
        n.push(".lock");
        let lock_name = Path::new(&n);
        let lock_guard = if lock_name.exists() {
            match Lock::exclusive(&lock_name) {
                Ok(x) => Some(x),
                Err(e) => {
                    error!("Failed to lock {:?}: {}, skipping", lock_name, e);
                    return Ok(());
                }
            }
        } else {
            None
        };
        debug!("Removing {:?}", path);
        clean_dir(path, remove_dir_itself)?;
        if let Some(_lock) = lock_guard {
            remove_file(lock_name)
                .map_err(|e| format!("Error removing lock file {:?}: {}",
                    lock_name, e))?;
        }
    }
    Ok(())
}

fn clean_everything(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        if let Some(ref cache_dir) = wrapper.ext_settings.cache_dir {
            clean_dir_wrapper(&cache_dir, false, dry_run)?;
        }
        if let Some(ref storage_dir) = wrapper.ext_settings.storage_dir {
            clean_dir_wrapper(&storage_dir, false, dry_run)?;
        }
    } else {
        let base = match setup::get_vagga_base(
            wrapper.project_root, wrapper.ext_settings)?
        {
            Some(base) => base,
            None => {
                warn!("No vagga directory exists");
                return Ok(());
            }
        };
        clean_dir_wrapper(&base, true, dry_run)?;
        let inner = wrapper.project_root.join(".vagga");
        if base != inner {
            clean_dir_wrapper(&inner, true, dry_run)?;
        }

    }
    return Ok(());
}

fn clean_temporary(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings)?
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
            clean_dir_wrapper(&entry.path(), true, dry_run)?;
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
    let base = match setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings)?
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

    info!("Useful images {:?}", useful);
    clean_dirs_except(&base.join(".roots"), &useful, dry_run)?;

    return Ok(());
}

fn clean_transient(wrapper: &Wrapper, global: bool, dry_run: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings)?
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
        clean_dir_wrapper(&entry.path(), true, dry_run)?;
    }

    return Ok(());
}

fn clean_dirs_except<P: AsRef<Path>>(roots: P, useful: &HashSet<String>,
    dry_run: bool)
    -> Result<(), String>
{
    let roots = roots.as_ref();
    for entry in try_msg!(read_dir(&roots),
                         "Can't read dir {dir:?}: {err}", dir=roots)
    {
        let entry = try_msg!(entry,
                             "Can't read dir {dir:?}: {err}", dir=roots);
        let path = entry.path();
        let typ = try_msg!(entry.file_type(),
            "Can't stat {p:?}: {err}", p=path);
        if !typ.is_dir() {
            if path.extension() == Some(OsStr::new("lock")) &&
               path.with_extension("").is_dir()
            {
                debug!("Skipping lock file {:?}", path);
            } else {
                match remove_file(&path) {
                    Ok(()) => {}
                    // File is deleted while we were scanning
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(ref e) => {
                        return Err(
                            format!("Can't remove file {:?}: {}", path, e));
                    }
                }
            }
        } else if !typ.is_dir() || entry.file_name()[..].to_str()
            .map(|n| !useful.contains(&n.to_string()))
            .unwrap_or(false)
        {
            clean_dir_wrapper(&entry.path(), true, dry_run)?;
        }
    }
    Ok(())
}

fn global_clean_unused(wrapper: &Wrapper, duration: Duration,
    dry_run: bool)
    -> Result<(), String>
{
    let unixtime = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let cut_off = (unixtime - duration).as_secs() as i64;
    let is_cache_dir = |p: &Path| {
        wrapper.ext_settings.cache_dir.as_ref()
        .map(|x| x == p)
        .unwrap_or(false)
    };
    let storage_dir = wrapper.ext_settings.storage_dir.as_ref().unwrap();
    let mut proj_num = 0;
    let mut to_remove = 0;
    let mut to_keep = 0;
    ScanDir::dirs().read(&storage_dir, |iter| {
        for (entry, name) in iter {
            let path = entry.path();
            if is_cache_dir(&path) {
                continue;
            }
            proj_num += 1;
            info!("Scanning project {}", name);

            let mut useful: HashSet<String> = HashSet::new();
            let roots = path.join(".roots");
            ScanDir::dirs().skip_hidden(false).read(&roots, |iter| {
                for (entry, name) in iter {
                    let luse_path = entry.path().join("last_use");
                    match metadata(&luse_path) {
                        Ok(ref meta) if meta.mtime() > cut_off => {
                            useful.insert(name);
                            to_keep += 1;
                        }
                        Ok(_) => {
                            to_remove += 1;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
                        Err(e) => {
                            error!("Error trying to stat {:?}: {}",
                                luse_path, e);
                        }
                    }
                }
            }).map_err(|e| {
                error!("Error reading {:?}: {}", roots, e);
            }).ok();

            info!("Useful images {:?}", useful);
            clean_dirs_except(&roots, &useful, dry_run)
            .map_err(|e| error!("Error cleaning {:?}: {}", roots, e))
            .ok(); // TODO(tailhook) propagate the errorneous exit code?
        }
    }).map_err(|e| {
        format!("Error reading storage dir {:?}: {}", storage_dir, e)
    })?;
    info!("Scanned {} projects, keeping {} images, removed {}",
        proj_num, to_keep, to_remove);
    Ok(())
}

fn clean_unused(wrapper: &Wrapper, duration: Option<Duration>,
    dry_run: bool)
    -> Result<(), String>
{

    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    let mut useful: HashSet<String> = HashSet::new();
    if let Some(duration) = duration {
        let unixtime = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let cut_off = (unixtime - duration).as_secs() as i64;
        ScanDir::dirs().skip_hidden(false).read("/vagga/base/.roots", |iter| {
            for (entry, name) in iter {
                let luse_path = entry.path().join("last_use");
                match metadata(&luse_path) {
                    Ok(ref meta) if meta.mtime() > cut_off => {
                        useful.insert(name);
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::NotFound => {}
                    Err(e) => {
                        error!("Error trying to stat {:?}: {}", luse_path, e);
                    }
                }
            }
        }).map_err(|e| {
            error!("Error reading `.vagga/.roots`: {}", e);
        }).ok();
    } else {
        for (name, _) in &wrapper.config.containers {
            if let Some(version) = get_version_hash(name, wrapper)? {
                useful.insert(format!("{}.{}", name, &version[..8]));
            }
        }
    }
    info!("Useful images {:?}", useful);
    clean_dirs_except("/vagga/base/.roots", &useful, dry_run)?;

    return Ok(());
}

fn clean_volumes(wrapper: &Wrapper, global: bool, dry_run: bool, all: bool)
    -> Result<(), String>
{
    if global {
        panic!("Global cleanup is not implemented yet");
    }
    let base = match setup::get_vagga_base(
        wrapper.project_root, wrapper.ext_settings)?
    {
        Some(base) => base,
        None => {
            warn!("No vagga directory exists");
            return Ok(());
        }
    };
    let volume_dir = base.join(".volumes");
    let mut useful = HashSet::new();
    if !all {
        for (_, container) in &wrapper.config.containers {
            for (_, vol) in &container.volumes {
                if let Persistent(ref p) = *vol {
                    useful.insert(p.name.clone());
                }
            }
        }
        for (_, command) in &wrapper.config.commands {
            match *command {
                Command(ref cmd) => {
                    for (_, vol) in &cmd.volumes {
                        if let Persistent(ref p) = *vol {
                            useful.insert(p.name.clone());
                        }
                    }
                }
                CapsuleCommand(_) => {
                    // novolumes
                },
                Supervise(ref cmd) => {
                    for (_, child) in &cmd.children {
                        for (_, vol) in child.get_volumes() {
                            if let Persistent(ref p) = *vol {
                                useful.insert(p.name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
    info!("Useful volumes {:?}", useful);
    clean_dirs_except(volume_dir, &useful, dry_run)?;

    return Ok(());
}
