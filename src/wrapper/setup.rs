use std::env;
use std::io::{BufRead, BufReader};
use std::env::{current_exe};
use std::fs::FileType::{Symlink, Directory};
use std::fs::FileType;
use std::ffi::CString;
use std::fs::File;
use std::fs::{create_dir, create_dir_all, copy, read_link, hard_link};
use std::os::unix::fs::symlink;
use std::fs::{remove_dir_all, read_dir};
use std::collections::BTreeMap;

use libc::chmod;
use libc::pid_t;

use config::Container;
use config::containers::Volume::{Tmpfs, VaggaBin, BindRW};
use container::root::{change_root};
use container::mount::{bind_mount, unmount, mount_system_dirs, remount_ro};
use container::mount::{mount_tmpfs, mount_pseudo};
use super::run::DEFAULT_PATH;
use settings::{MergedSettings};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    ReadOnly,
    TransientHardlinkCopy(pid_t),
}

fn create_storage_dir(storage_dir: &Path, project_root: &Path)
    -> Result<Path, String>
{
    let name = match project_root.filename_str() {
        Some(name) => name,
        None => return Err(format!(
            "Project dir `{}` is either root or has bad characters",
            project_root.display())),
    };
    let path = storage_dir.join(name);
    if !path.exists() {
        return Ok(path);
    }
    for i in range(1isize, 101isize) {
        let result = format!("{}-{}", name, i);
        let path = storage_dir.join(result);
        if !path.exists() {
            return Ok(path);
        }
    }
    return Err(format!("Too many similar paths named {} in {}",
        name, storage_dir.display()));
}


fn make_cache_dir(_project_root: &Path, vagga_base: &Path,
    settings: &MergedSettings)
    -> Result<Path, String>
{
    match settings.cache_dir {
        Some(ref dir) if settings.shared_cache => {
            if !dir.exists() {
                return Err(format!(concat!("Cache directory `{}` must exists.",
                    " Please either create it or remove that configuration",
                    " setting"), dir.display()));
            }
            return Ok(dir.clone());
        }
        _ => {
            let dir = vagga_base.join(".cache");
            try!(safe_ensure_dir(&dir));
            return Ok(dir);
        }
   }
}

fn safe_ensure_dir(dir: &Path) -> Result<(), String> {
    match dir.lstat() {
        Ok(stat) if stat.kind == Symlink => {
            return Err(format!(concat!("The `{0}` dir can't be a symlink. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Ok(stat) if stat.kind == Directory => {
            // ok
        }
        Ok(_) => {
            return Err(format!(concat!("The `{0}` must be a directory. ",
                               "Please run `unlink {0}`"), dir.display()));
        }
        Err(ref e) if e.kind == FileNotFound => {
            try!(mkdir(dir, ALL_PERMISSIONS)
                .map_err(|e| format!("Can't create `{}`: {}",
                                     dir.display(), e)));
        }
        Err(ref e) => {
            return Err(format!("Can't stat `{}`: {}", dir.display(), e));
        }
    }
    return Ok(());
}

fn _vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Result<Path, (Path, Path)>, String>
{
    if let Some(ref dir) = settings.storage_dir {
        let lnkdir = project_root.join(".vagga/.lnk");
        match read_link(&lnkdir) {
            Ok(lnk) => {
                if let Some(name) = lnk.filename() {
                    let target = dir.join(name);
                    if Path::new(lnk.dirname()) != *dir {
                        return Err(concat!("You have set storage_dir to {}, ",
                            "but .vagga/.lnk points to {}. You probably need ",
                            "to run `ln -sfn {} .vagga/.lnk`").to_string());
                    }
                    if !lnkdir.exists() {
                        return Err(format!("Your .vagga/.lnk points to a \
                            non-existent directory. Presumably you deleted \
                            dir {}. Just remove .vagga/.lnk now.",
                            lnk.display()));
                    }
                    return Ok(Ok(target));
                } else {
                    return Err(format!(concat!("Bad link .vagga/.lnk: {}.",
                        " You are pobably need to remove it now"),
                        lnk.display()));
                }
            }
            Err(ref e) if e.kind == FileNotFound => {
                return Ok(Err((lnkdir, dir.clone())));
            }
            Err(ref e) => {
                return Err(format!("Can't read link .vagga/.lnk: {}", e));
            }
        };
    } else {
        return Ok(Ok(project_root.join(".vagga")));
    }
}

pub fn get_vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Option<Path>, String>
{
    return _vagga_base(project_root, settings).map(|x| x.ok());
}

fn vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Path, String>
{
    match _vagga_base(project_root, settings) {
        Ok(Err((lnkdir, dir))) => {
            let target = try!(create_storage_dir(&dir, project_root));
            try!(safe_ensure_dir(&target));
            try!(symlink(&target, &lnkdir)
                .map_err(|e| format!("Error symlinking storage: {}", e)));
            try!(symlink(project_root, &target.join(".lnk"))
                .map_err(|e| format!("Error symlinking storage: {}", e)));
            return Ok(target)
        }
        Ok(Ok(path)) => {
            return Ok(path);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

fn make_mountpoint(project_root: &Path) -> Result<(), String> {
    let vagga_dir = project_root.join(".vagga");
    try!(safe_ensure_dir(&vagga_dir));
    let mnt_dir = vagga_dir.join(".mnt");
    try!(safe_ensure_dir(&mnt_dir));
    return Ok(());
}

pub fn setup_base_filesystem(project_root: &Path, settings: &MergedSettings)
    -> Result<(), String>
{
    let mnt_dir = project_root.join(".vagga/.mnt");
    try!(make_mountpoint(project_root));
    try!(mount_tmpfs(&mnt_dir, "size=100m"));

    let proc_dir = mnt_dir.join("proc");
    try!(mkdir(&proc_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /proc: {}", e)));
    try!(mount_pseudo(&proc_dir, "proc", "", false));

    let dev_dir = mnt_dir.join("dev");
    try!(mkdir(&dev_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /dev: {}", e)));
    try!(bind_mount(&Path::new("/dev"), &dev_dir));

    let sys_dir = mnt_dir.join("sys");
    try!(mkdir(&sys_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /sys: {}", e)));
    try!(bind_mount(&Path::new("/sys"), &sys_dir));

    let vagga_dir = mnt_dir.join("vagga");
    try!(mkdir(&vagga_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /vagga: {}", e)));

    let bin_dir = vagga_dir.join("bin");
    try!(mkdir(&bin_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /vagga/bin: {}", e)));
    try!(bind_mount(&self_exe_path().unwrap(), &bin_dir));
    try!(remount_ro(&bin_dir));

    let etc_dir = mnt_dir.join("etc");
    try!(mkdir(&etc_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /etc: {}", e)));
    try!(copy(&Path::new("/etc/hosts"), &etc_dir.join("hosts"))
        .map_err(|e| format!("Error copying /etc/hosts: {}", e)));
    try!(copy(&Path::new("/etc/resolv.conf"), &etc_dir.join("resolv.conf"))
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));

    let local_base = vagga_dir.join("base");
    try!(safe_ensure_dir(&local_base));
    let vagga_base = try!(vagga_base(project_root, settings));
    try!(bind_mount(&vagga_base, &local_base));
    try!(safe_ensure_dir(&local_base.join(".roots")));
    try!(safe_ensure_dir(&local_base.join(".transient")));

    let cache_dir = vagga_dir.join("cache");
    try!(mkdir(&cache_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /vagga/cache: {}", e)));
    let locl_cache = try!(make_cache_dir(project_root, &vagga_base, settings));
    try!(bind_mount(&locl_cache, &cache_dir));

    if let Some(nsdir) = getenv("VAGGA_NAMESPACE_DIR") {
        let newns_dir = vagga_dir.join("namespaces");
        try!(mkdir_recursive(&newns_dir, ALL_PERMISSIONS)
            .map_err(|e| format!("Error creating directory \
                for namespaces: {}", e)));
        try!(bind_mount(&Path::new(nsdir), &newns_dir)
             .map_err(|e| format!("Error mounting directory \
                with namespaces: {}", e)));
    }

    let work_dir = mnt_dir.join("work");
    try!(mkdir(&work_dir, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /work: {}", e)));
    try!(bind_mount(project_root, &work_dir));


    let old_root = vagga_dir.join("old_root");
    try!(mkdir(&old_root, ALL_PERMISSIONS)
        .map_err(|e| format!("Error creating /vagga/old_root: {}", e)));
    try!(change_root(&mnt_dir, &old_root));
    try!(unmount(&Path::new("/vagga/old_root")));

    Ok(())
}

pub fn get_environment(container: &Container)
    -> Result<BTreeMap<String, String>, String>
{
    let mut result = BTreeMap::new();
    result.insert("TERM".to_string(),
                  getenv("TERM").unwrap_or("dumb".to_string()));
    result.insert("PATH".to_string(),
                  DEFAULT_PATH.to_string());
    for (k, v) in env().into_iter() {
        if k.starts_with("VAGGAENV_") {
            result.insert(k.slice_from(9).to_string(), v);
        }
    }
    if let Some(ref filename) = container.environ_file {
        let mut f = BufferedReader::new(try!(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e))));
        for line_read in f.lines() {
            let line = try!(line_read
                .map_err(|e| format!("Error reading environment file {}: {}",
                    filename.display(), e)));
            let mut pair = line.as_slice().splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = try!(pair.next()
                .ok_or(format!("Error reading environment file {}: bad format",
                    filename.display())));
            if value.len() > 0 && value.starts_with("\"") {
                value = value.trim_matches('"');
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    for (ref k, ref v) in container.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    return Ok(result);
}

fn hardlink_dir(old: &Path, new: &Path) -> Result<(), String> {
    let filelist = try!(readdir(old)
        .map_err(|e| format!("Error reading directory: {}", e)));
    for item in filelist.iter() {
        let stat = try!(item.lstat()
            .map_err(|e| format!("Error stat for file: {}", e)));
        let nitem = new.join(item.filename().unwrap());
        match stat.kind {
            FileType::RegularFile => {
                try!(link(item, &nitem)
                    .map_err(|e| format!("Can't hard-link file: {}", e)));
            }
            FileType::Directory => {
                try!(mkdir(&nitem, ALL_PERMISSIONS)
                    .map_err(|e| format!("Can't create dir: {}", e)));
                try!(chmod(&nitem, stat.perm)
                    .map_err(|e| format!("Can't chmod: {}", e)));
                try!(hardlink_dir(item, &nitem));
            }
            FileType::NamedPipe => {
                warn!("Skipping named pipe {:?}", item);
            }
            FileType::BlockSpecial => {
                warn!("Can't clone block-special {:?}, skipping", item);
            }
            FileType::Symlink => {
                let lnk = try!(readlink(item)
                    .map_err(|e| format!("Can't readlink: {}", e)));
                try!(symlink(&lnk, &nitem)
                    .map_err(|e| format!("Can't symlink: {}", e)));
            }
            FileType::Unknown => {
                warn!("Unknown file type {:?}", item);
            }
        }
    }
    Ok(())
}

pub fn setup_filesystem(container: &Container, write_mode: WriteMode,
    container_ver: &str)
    -> Result<(), String>
{
    let root_path = Path::new("/");
    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try!(mkdir(&tgtroot, ALL_PERMISSIONS)
             .map_err(|x| format!("Error creating directory: {}", x)));
    }
    match write_mode {
        WriteMode::ReadOnly => {
            let nroot = Path::new("/vagga/base/.roots")
                .join(container_ver).join("root");
            try!(bind_mount(&nroot, &tgtroot)
                 .map_err(|e| format!("Error bind mount: {}", e)));
        }
        WriteMode::TransientHardlinkCopy(pid) => {
            let oldpath = Path::new("/vagga/base/.roots")
                .join(container_ver).join("root");
            let newpath = Path::new("/vagga/base/.transient")
                .join(format!("{}.{}", container_ver, pid));
            if newpath.exists() {
                try!(rmdir_recursive(&newpath)
                    .map_err(|e| format!("Error removing dir: {}", e)));
            }
            try!(mkdir(&newpath, ALL_PERMISSIONS)
                .map_err(|e| format!("Error creating directory: {}", e)));
            try!(hardlink_dir(&oldpath, &newpath));
            try!(bind_mount(&newpath, &tgtroot)
                 .map_err(|e| format!("Error bind mount: {}", e)));
        }
    };

    try!(mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e)));

    if let None = container.volumes.get(&Path::new("/tmp")) {
        try!(mount_tmpfs(&tgtroot.join("tmp"), "size=100m,mode=01777"));
    }

    for (path, vol) in container.volumes.iter() {
        let dest = tgtroot.join(path.path_relative_from(&root_path).unwrap());
        match vol {
            &Tmpfs(ref params) => {
                try!(mount_tmpfs(&dest,
                    format!("size={},mode=0{:o}", params.size, params.mode)
                    .as_slice()));
            }
            &VaggaBin => {
                try!(bind_mount(&Path::new("/vagga/bin"), &dest));
            }
            &BindRW(ref bindpath) => {
                let src = tgtroot.join(
                    &bindpath.path_relative_from(&root_path).unwrap());
                try!(bind_mount(&src, &dest));
            }
        }
    }
    if let Some(_) = getenv("VAGGA_NAMESPACE_DIR") {
        let newns_dir = tgtroot.join("tmp/vagga/namespaces");
        try!(mkdir_recursive(&newns_dir, ALL_PERMISSIONS)
            .map_err(|e| format!("Error creating directory \
                for namespaces: {}", e)));
        try!(bind_mount(&Path::new("/vagga/namespaces"), &newns_dir)
             .map_err(|e| format!("Error mounting directory \
                with namespaces: {}", e)));
    }

    if let Some(ref path) = container.resolv_conf_path {
        let path = tgtroot.join(path.path_relative_from(&root_path).unwrap());
        try!(copy(&Path::new("/etc/resolv.conf"), &path)
            .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e)));
    }

    //  Currently we need the root to be writeable for putting resolv.conf
    //  It's a bit ugly but bearable for development environments.
    //  Eventually we'll find a better way
    if write_mode == WriteMode::ReadOnly {
        try!(remount_ro(&tgtroot));
    }

    try!(change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e)));
    try!(unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e)));
    try!(unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting `/tmp`: {}", e)));
    Ok(())
}

