use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::env::{current_exe};
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::fs::{read_link, OpenOptions};
use std::fs::File;
use std::os::unix::fs::{symlink};
use std::path::{Path, PathBuf};

use libc::pid_t;
use libmount::{BindMount, Tmpfs, Remount};

use config::{Container, Settings};
use config::command::{CommandInfo, CapsuleInfo};
use config::volumes::Volume;
use config::volumes::Volume as V;
use container::root::{change_root};
use container::mount::{unmount, mount_system_dirs};
use container::mount::{mount_proc, mount_dev, mount_run};
use container::util::{hardlink_dir, clean_dir};
use container::network::detect_local_dns;
use config::read_settings::{MergedSettings};
use process_util::{DEFAULT_PATH, PROXY_ENV_VARS};
use file_util::{Dir, safe_ensure_dir};
use wrapper::snapshot::make_snapshot;
use container::util::version_from_symlink;
use storage_dir::sanitize;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteMode {
    ReadOnly,
    TransientHardlinkCopy(pid_t),
}

fn create_storage_dir(storage_dir: &Path, project_root: &Path)
    -> Result<PathBuf, String>
{
    let name = match project_root.file_name().and_then(|x| x.to_str()) {
        Some(name) => name,
        None => return Err(format!(
            "Project dir {:?} is either root or has bad characters",
            project_root)),
    };
    let path = storage_dir.join(name);
    if !path.exists() {
        return Ok(path);
    }
    for i in 1..101 {
        let result = format!("{}-{}", name, i);
        let path = storage_dir.join(result);
        if !path.exists() {
            return Ok(path);
        }
    }
    return Err(format!("Too many similar paths named {} in {:?}",
        name, storage_dir));
}


fn make_cache_dir(_project_root: &Path, vagga_base: &Path,
    settings: &MergedSettings)
    -> Result<PathBuf, String>
{
    match settings.cache_dir {
        Some(ref dir) if settings.shared_cache => {
            if !dir.exists() {
                return Err(format!(concat!("Cache directory {:?} must exists.",
                    " Please either create it or remove that configuration",
                    " setting"), dir));
            }
            return Ok(dir.clone());
        }
        _ => {
            let dir = vagga_base.join(".cache");
            safe_ensure_dir(&dir)?;
            return Ok(dir);
        }
   }
}

fn _vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<Result<PathBuf, (PathBuf, PathBuf)>, String>
{
    if let Some(ref dir) = settings.storage_dir {
        let lnkdir = project_root.join(".vagga/.lnk");
        match read_link(&lnkdir) {
            Ok(lnk) => {
                if let Some(name) = lnk.file_name() {
                    let target = dir.join(name);
                    if lnk.parent() != Some(&*dir) {
                        let newlnk = dir.join(name);
                        if newlnk.exists() {
                            return Err(format!("You have set `storage-dir` \
                                to {:?}, but .vagga/.lnk points to {:?}. \
                                You probably need to run \
                                `ln -sfn {:?} .vagga/.lnk`",
                                dir, lnk, newlnk));
                        } else {
                            return Err(format!("You have set `storage-dir` \
                                to {:?}, but .vagga/.lnk points to {:?}. \
                                If storage dir is new, \
                                just remove `.vagga` dir.",
                                dir, lnk));
                        }
                    }
                    if !lnkdir.exists() {
                        return Err(format!("Your .vagga/.lnk points to a \
                            non-existent directory. Presumably you deleted \
                            dir {:?}. Just remove .vagga/.lnk now.",
                            lnk));
                    }
                    return Ok(Ok(target));
                } else {
                    return Err(format!(concat!("Bad link .vagga/.lnk: {:?}.",
                        " You are pobably need to remove it now"),
                        lnk));
                }
            }
            Err(ref e) if e.kind() == ErrorKind::NotFound => {
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
    -> Result<Option<PathBuf>, String>
{
    return _vagga_base(project_root, settings).map(|x| x.ok());
}

fn vagga_base(project_root: &Path, settings: &MergedSettings)
    -> Result<PathBuf, String>
{
    match _vagga_base(project_root, settings) {
        Ok(Err((lnkdir, dir))) => {
            let ref svar = settings.storage_subdir_from_env_var;
            let target = if let Some(ref var) = *svar {
                if let Ok(user_dir) = env::var("_VAGGA_STORAGE_SUBDIR") {
                    let sanitized = sanitize(&user_dir);
                    if sanitized.len() == 0 {
                        return Err(format!("Couldn't determine storage-dir \
                            subdir from environment, original value {:?} \
                            (got from {:?})",
                            user_dir, var));
                    } else {
                        info!("Storage dir {:?} (got from {:?})",
                            user_dir, var);
                        dir.join(user_dir)
                    }
                } else {
                    return Err("Internal error: _VAGGA_STORAGE_SUBDIR".into())
                }
            } else {
                create_storage_dir(&dir, project_root)?
            };
            safe_ensure_dir(&target)?;
            symlink(&target, &lnkdir)
                .map_err(|e| format!("Error symlinking storage: {}", e))?;
            if svar.is_none() {
                // Backlink is useful only when not using from-env-var
                symlink(project_root, &target.join(".lnk"))
                    .map_err(|e| format!("Error symlinking storage: {}", e))?;
            }
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
    safe_ensure_dir(&vagga_dir)?;
    let mnt_dir = vagga_dir.join(".mnt");
    safe_ensure_dir(&mnt_dir)?;
    return Ok(());
}

fn env_is_set(key: &str) -> bool {
    env::var(key).map(|x| x.len() > 0).unwrap_or(false)
}

pub fn setup_base_filesystem(project_root: &Path, settings: &MergedSettings)
    -> Result<(), String>
{
    let mnt_dir = project_root.join(".vagga/.mnt");
    make_mountpoint(project_root)?;
    Tmpfs::new(&mnt_dir).size_bytes(100 << 20).mount()
         .map_err(|e| format!("{}", e))?;

    let proc_dir = mnt_dir.join("proc");
    try_msg!(Dir::new(&proc_dir).create(),
             "Error creating /proc: {err}");
    mount_proc(&proc_dir)?;

    let dev_dir = mnt_dir.join("dev");
    try_msg!(Dir::new(&dev_dir).create(),
             "Error creating /dev: {err}");
    mount_dev(&dev_dir)?;

    let sys_dir = mnt_dir.join("sys");
    try_msg!(Dir::new(&sys_dir).create(),
             "Error creating /sys: {err}");
    try_msg!(BindMount::new("/sys", &sys_dir).mount(), "mount /sys: {err}");
    let selinux = sys_dir.join("fs/selinux");
    if selinux.is_dir() {
        // Need this go get some selinux-aware commands to work (see #65)
        Remount::new(&sys_dir.join("fs/selinux"))
            .bind(true).readonly(true).remount()
            .map_err(|e| warn!("remount /sys/fs/selinux: {}", e)).ok();
    }

    let vagga_dir = mnt_dir.join("vagga");
    try_msg!(Dir::new(&vagga_dir).create(),
             "Error creating /vagga: {err}");

    let bin_dir = vagga_dir.join("bin");
    try_msg!(Dir::new(&bin_dir).create(),
             "Error creating /vagga/bin: {err}");
    try_msg!(BindMount::new(&current_exe().unwrap().parent().unwrap(),
                            &bin_dir)
             .readonly(true)
             .mount(), "mount /vagga/bin: {err}");

    let etc_dir = mnt_dir.join("etc");
    try_msg!(Dir::new(&etc_dir).create(),
             "Error creating /etc: {err}");
    let ref hosts_path = etc_dir.join("hosts");
    File::create(hosts_path).map_err(|e|
        format!("Cannot create {:?} file: {}", hosts_path, e))?;
    BindMount::new(Path::new("/etc/hosts"), hosts_path)
        .mount()
        .map_err(|e| format!("Error mounting /etc/hosts: {}", e))?;

    let local_dns = if env_is_set("VAGGA_IN_NAMESPACE") ||
                       env_is_set("VAGGA_NAMESPACE_DIR")
    {
        detect_local_dns().map_err(|e| format!("{}", e))?
    } else {
        None
    };
    let resolv_source = if let Some(_) = local_dns {
        // TODO(tailhook) do it on detached tmpfs
        File::create("/tmp/resolv.conf")
            .and_then(|mut f|
                // redirect to a gateway IP
                write!(&mut f, "nameserver 172.23.255.1"))
            .map_err(|e| format!("Can't create temporary resolv.conf: {}", e))?;
        Path::new("/tmp/resolv.conf")
    } else {
        Path::new("/etc/resolv.conf")
    };
    let ref resolv_path = etc_dir.join("resolv.conf");
    if !resolv_path.exists() {
        File::create(resolv_path).map_err(|e|
            format!("Cannot create {:?} file: {}", resolv_path, e))?;
    }
    BindMount::new(&resolv_source, resolv_path)
        .mount()
        .map_err(|e| format!("Error copying /etc/resolv.conf: {}", e))?;

    let local_base = vagga_dir.join("base");
    safe_ensure_dir(&local_base)?;
    let vagga_base = vagga_base(project_root, settings)?;

    try_msg!(BindMount::new(&vagga_base, &local_base).mount(),
        "mount /vagga/base: {err}");
    safe_ensure_dir(&local_base.join(".roots"))?;
    safe_ensure_dir(&local_base.join(".transient"))?;

    let cache_dir = vagga_dir.join("cache");
    try_msg!(Dir::new(&cache_dir).create(),
        "Error creating /vagga/cache: {err}");
    let locl_cache = make_cache_dir(project_root, &vagga_base, settings)?;
    try_msg!(BindMount::new(&locl_cache, &cache_dir).mount(),
        "mount /vagga/cache: {err}");

    if let Ok(nsdir) = env::var("VAGGA_NAMESPACE_DIR") {
        let newns_dir = vagga_dir.join("namespaces");
        try_msg!(Dir::new(&newns_dir).recursive(true).create(),
            "Error creating directory for namespaces: {err}");
        try_msg!(BindMount::new(&nsdir, &newns_dir).mount(),
             "namespace dir: {err}");
    }

    let volume_dir = mnt_dir.join("volumes");
    try_msg!(Dir::new(&volume_dir).create(),
        "Error creating /volumes: {err}");
    for (name, source_path) in &settings.external_volumes {
        let dest = volume_dir.join(name);
        if source_path.is_dir() {
            try_msg!(Dir::new(&dest).create(),
                "Error creating {p:?}: {err}", p=dest);
        } else {
            try_msg!(File::create(&dest),
                "Error creating {p:?}: {err}", p=dest);
        }
        try_msg!(BindMount::new(source_path, &dest).mount(),
            "volume: {err}");
    }

    let work_dir = mnt_dir.join("work");
    try_msg!(Dir::new(&work_dir).create(),
        "Error creating /work: {err}");
    try_msg!(BindMount::new(project_root, &work_dir).mount(),
        "mount /work: {err}");


    let old_root = vagga_dir.join("old_root");
    try_msg!(Dir::new(&old_root).create(),
             "Error creating /vagga/old_root: {err}");
    change_root(&mnt_dir, &old_root)?;
    unmount(&Path::new("/work/.vagga/.mnt"))
         .map_err(|e| format!("Error unmounting `.vagga/.mnt`: {}", e))?;
    unmount(&Path::new("/vagga/old_root"))?;

    Ok(())
}

pub fn get_environment(settings: &Settings, container: &Container,
    command: Option<&CommandInfo>)
    -> Result<BTreeMap<String, String>, String>
{
    _get_environment(DEFAULT_PATH, settings, Some(container),
        command.map(|c| &c.environ))
}

pub fn get_capsule_environment(settings: &Settings,
    command: &CapsuleInfo)
    -> Result<BTreeMap<String, String>, String>
{
    _get_environment("/vagga/bin:/bin", settings, None, Some(&command.environ))
}

fn _get_environment(default_path: &str, settings: &Settings,
    container: Option<&Container>,
    command_env: Option<&BTreeMap<String, String>>)
    -> Result<BTreeMap<String, String>, String>
{
    let mut result = BTreeMap::new();
    result.insert("PATH".to_string(),
                  default_path.to_string());
    if settings.proxy_env_vars {
        for k in &PROXY_ENV_VARS {
            if let Ok(v) = env::var(k) {
                result.insert(k.to_string(), v);
            }
        }
    }
    if let Some(ref filename) = container.and_then(|x| x.environ_file.as_ref()) {
        let f = BufReader::new(
                File::open(filename)
                .map_err(|e| format!("Error reading environment file {:?}: {}",
                    filename, e))?);
        for line_read in f.lines() {
            let line = line_read
                .map_err(|e| format!("Error reading environment file {:?}: {}",
                    filename, e))?;
            let mut pair = line[..].splitn(2, '=');
            let key = pair.next().unwrap();
            let mut value = pair.next()
                .ok_or(format!("Error reading environment file {:?}: bad format",
                    filename))?;
            if value.len() > 0 && value.starts_with("\"") {
                value = value.trim_matches('"');
            }
            result.insert(key.to_string(), value.to_string());
        }
    }
    if let Some(container) = container {
        for (ref k, ref v) in container.environ.iter() {
            result.insert(k.to_string(), v.to_string());
        }
    }
    if let Some(command_env) = command_env {
        for (ref k, ref v) in command_env.iter() {
            result.insert(k.to_string(), v.to_string());
        }
    }
    for (ref k, ref v) in settings.environ.iter() {
        result.insert(k.to_string(), v.to_string());
    }
    for (k, v) in env::vars() {
        if k.starts_with("VAGGAENV_") {
            result.insert(k[9..].to_string(), v);
        } else if k.starts_with("VAGGAOPT_") || k.starts_with("VAGGACLI_") {
            result.insert(k, v);
        }
    }
    return Ok(result);
}

pub struct SetupInfo<'a> {
    pub volumes: BTreeMap<&'a PathBuf, &'a Volume>,
    pub tmp_volumes: BTreeSet<&'a String>,
    pub write_mode: WriteMode,
    pub resolv_conf_path: Option<&'a PathBuf>,
    pub hosts_file_path: Option<&'a PathBuf>,
}

impl<'a> SetupInfo<'a> {
    pub fn from_container(container: &Container) -> SetupInfo {
        let mut setup_info = SetupInfo {
            volumes: BTreeMap::new(),
            tmp_volumes: BTreeSet::new(),
            write_mode: WriteMode::ReadOnly,
            resolv_conf_path: container.resolv_conf_path.as_ref(),
            hosts_file_path: container.hosts_file_path.as_ref(),
        };
        setup_info.volumes(&container.volumes);
        setup_info
    }
    pub fn volumes(&mut self, volumes: &'a BTreeMap<PathBuf, Volume>)
        -> &mut SetupInfo<'a>
    {
        for (path, vol) in volumes.iter() {
            self.volumes.insert(&path, &vol);
        }
        self
    }
    pub fn write_mode(&mut self, write_mode: WriteMode)
        -> &mut SetupInfo<'a>
    {
        self.write_mode = write_mode;
        self
    }
}

pub fn setup_filesystem(setup_info: &SetupInfo, container_ver: &str)
    -> Result<(), String>
{
    let tgtroot = Path::new("/vagga/root");
    if !tgtroot.exists() {
        try_msg!(Dir::new(&tgtroot).create(),
                 "Can't create rootfs mountpoint: {err}");
    }
    let image_base = Path::new("/vagga/base/.roots").join(container_ver);
    match setup_info.write_mode {
        WriteMode::ReadOnly => {
            let nroot = image_base.join("root");
            try_msg!(BindMount::new(&nroot, &tgtroot).readonly(true).mount(),
                 "mount root: {err}");
        }
        WriteMode::TransientHardlinkCopy(pid) => {
            let oldpath = image_base.join("root");
            let newpath = Path::new("/vagga/base/.transient")
                .join(format!("{}.{}", container_ver, pid));
            if newpath.exists() {
                try_msg!(clean_dir(&newpath, true),
                        "Error removing dir: {err}");
            }
            try_msg!(Dir::new(&newpath).create(),
                     "Error creating directory: {err}");
            try_msg!(hardlink_dir(&oldpath, &newpath),
                "Can't hardlink {p:?}: {err}", p=newpath);
            try_msg!(BindMount::new(&newpath, &tgtroot).mount(),
                 "transient root: {err}");
        }
    };
    File::create(image_base.join("last_use"))
        .map_err(|e| warn!("Can't write image usage info: {}", e)).ok();

    mount_system_dirs()
        .map_err(|e| format!("Error mounting system dirs: {}", e))?;

    if let None = setup_info.volumes.get(&PathBuf::from("/tmp")) {
        Tmpfs::new(&tgtroot.join("tmp"))
            .size_bytes(100 << 20)
            .mode(0o1777)
            .mount().map_err(|e| format!("{}", e))?;
    }
    if let None = setup_info.volumes.get(&PathBuf::from("/run")) {
        mount_run(&tgtroot.join("run"))?;
    }

    for (path, vol) in setup_info.volumes.iter() {
        let ref rel_path = path.strip_prefix("/").unwrap();
        let dest = tgtroot.join(rel_path);
        match *vol {
            &V::Tmpfs(ref params) => {
                Tmpfs::new(&dest)
                    .size_bytes(params.size)
                    .mode(params.mode)
                    .mount().map_err(|e| format!("{}", e))?;
                for (subpath, info) in &params.subdirs {
                    try_msg!(Dir::new(&dest.join(&subpath))
                            .mode(info.mode)
                            .create(),
                        "Error creating subdir {sub:?} of {vol:?}: {err}",
                        sub=subpath, vol=path);
                }
                for (subpath, text) in &params.files {
                    let text = text.as_ref().map(|x| &x[..]).unwrap_or("");
                    try_msg!(File::create(&dest.join(&subpath))
                        .and_then(|mut f| f.write_all(text.as_bytes())),
                        "Error creating file {sub:?} of {vol:?}: {err}",
                        sub=subpath, vol=path);
                }
            }
            &V::VaggaBin => {
                try_msg!(BindMount::new("/vagga/bin", &dest).mount(),
                    "mount !VaggaBin: {err}");
            }
            &V::BindRW(ref bindpath) => {
                try_msg!(BindMount::new(&bindpath, &dest).mount(),
                    "mount !BindRW: {err}");
            }
            &V::BindRO(ref bindpath) => {
                try_msg!(BindMount::new(&bindpath, &dest)
                        .readonly(true)
                        .mount(),
                    "mount !BindRO: {err}");
            }
            &V::CacheDir(ref bindpath) => {
                // avoid mounting cache root
                if bindpath == Path::new("") {
                    return Err("mount !CacheDir: path must not be empty".into())
                }

                // bind path must be relative to cache root
                if bindpath.is_absolute() {
                    return Err(format!(
                        "mount !CacheDir {:?}: path must not be absolute",
                        &bindpath))
                }

                let source = Path::new("/vagga/cache").join(bindpath);
                if !source.exists() {
                    try_msg!(Dir::new(&source).recursive(true).create(),
                        "error creating dir for volume {p:?}: {err}", p=path);
                }
                try_msg!(BindMount::new(&source, &dest).mount(),
                    "mount !CacheDir: {err}");
            }
            &V::Empty => {
                Tmpfs::new(&dest)
                    .size_bytes(1)
                    .mode(0)
                    .mount().map_err(|e| format!("{}", e))?;
                try_msg!(Remount::new(&dest).bind(true).readonly(true).remount(),
                    "remount !Empty: {err}");
            }
            &V::Snapshot(ref info) => {
                let ref src = match info.container {
                    Some(ref src_container) => {
                        let container_ver = version_from_symlink(
                            format!("/work/.vagga/{}", src_container))?;
                        Path::new("/vagga/base/.roots")
                            .join(container_ver)
                            .join("root")
                            .join(rel_path)
                    },
                    None => dest.clone(),
                };
                make_snapshot(src, &dest, info)?;
            }
            &V::Container(ref child_cont) => {
                let container_ver = version_from_symlink(
                    format!("/work/.vagga/{}", child_cont))?;
                let target = Path::new("/vagga/base/.roots")
                    .join(container_ver).join("root");
                try_msg!(BindMount::new(&target, &dest).readonly(true).mount(),
                    "mount !Container: {err}");
            }
            &V::Persistent(ref info) => {
                if setup_info.tmp_volumes.contains(&info.name) {
                    let path = Path::new("/vagga/base/.volumes")
                        .join(&format!(".tmp.{}", info.name));
                    try_msg!(Dir::new(&path).recursive(true).create(),
                        "error creating dir for volume {p:?}: {err}", p=path);
                    try_msg!(BindMount::new(&path, &dest).mount(),
                        "mount !BindRW: {err}");
                } else {
                    let path = Path::new("/vagga/base/.volumes")
                        .join(&info.name);
                    if info.init_command.is_none() {
                        try_msg!(Dir::new(&path)
                                .recursive(true)
                                .uid(info.owner_uid)
                                .gid(info.owner_gid)
                                .create(),
                            "error creating dir for volume {p:?}: {err}",
                            p=path);
                    } else if !path.exists() {
                        return Err(format!("Internal error: \
                            Got uninitialized volume {:?}",
                            info.name));
                    }
                    try_msg!(BindMount::new(&path, &dest).mount(),
                        "mount !BindRW: {err}");
                };
            }
        }
    }
    if let Ok(_) = env::var("VAGGA_NAMESPACE_DIR") {
        let newns_dir = tgtroot.join("tmp/vagga/namespaces");
        try_msg!(Dir::new(&newns_dir).recursive(true).create(),
            "Error creating directory for namespaces: {err}");
        try_msg!(BindMount::new("/vagga/namespaces", &newns_dir).mount(),
            "mount namespaces: {err}");
    }

    if let Some(path) = setup_info.resolv_conf_path {
        if !setup_info.volumes.contains_key(&path) {
            let ref path = tgtroot.join(path.strip_prefix("/").unwrap());
            create_and_mount(Path::new("/etc/resolv.conf"), path)?;
        }
    }
    if let Some(path) = setup_info.hosts_file_path {
        if !setup_info.volumes.contains_key(&path) {
            let ref path = tgtroot.join(path.strip_prefix("/").unwrap());
            create_and_mount(Path::new("/etc/hosts"), path)?;
        }
    }

    change_root(&tgtroot, &tgtroot.join("tmp"))
         .map_err(|e| format!("Error changing root: {}", e))?;
    unmount(&Path::new("/tmp"))
         .map_err(|e| format!("Error unmounting `/tmp`: {}", e))?;
    Ok(())
}

fn create_and_mount(src: &Path, dst: &Path) -> Result<(), String> {
    OpenOptions::new().write(true).create_new(true).open(dst)
        .map(|_| ())
        .or_else(|e| {
            if e.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(format!("Cannot create file {:?}: {}", dst, e))
            }
        })?;
    BindMount::new(src, dst)
        .mount()
        .map_err(|e| format!("Error mounting {:?}: {}", src, e))?;
    Ok(())
}
