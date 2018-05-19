use std::env;
use std::fs::File;
use std::fs::{read_link, rename, remove_dir, remove_file};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::io::{stdout, stderr};
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::os::unix::fs::symlink;
use std::os::unix::io::FromRawFd;

use argparse::{ArgumentParser, Store, StoreTrue};
use serde_json;
use unshare::{Command, Namespace, ExitStatus};
use libmount::BindMount;
use itertools::Itertools;

use builder::context::Context;
use builder::commands::tarcmd::unpack_file;
use capsule::download::maybe_download_and_check_hashsum;
use config::{Config, Container, Settings};
use container::util::{clean_dir, hardlink_container_files};
use container::util::write_container_signature;
use container::util::{collect_container_dirs, collect_containers_from_storage};
use container::mount::{unmount};
use file_util::{Dir, Lock, copy, human_size};
use process_util::{capture_fd3_status, copy_env_vars};
use super::Wrapper;
use super::setup;
use build_step::Step;
use options::version_hash::Options;


struct ContainerInfo<'a> {
    pub name: &'a str,
    pub container: &'a Container,
    pub tmp_root_dir: PathBuf,
    pub force: bool,
    pub no_image: bool,
}

impl<'a> ContainerInfo<'a> {
    pub fn new(name: &'a str, container: &'a Container,
        force: bool, no_image: bool)
        -> ContainerInfo<'a>
    {
        let tmp_root_dir = PathBuf::from(
            &format!("/vagga/base/.roots/.tmp.{}", name));
        ContainerInfo {
            name: name,
            container: container,
            tmp_root_dir: tmp_root_dir,
            force: force,
            no_image: no_image,
        }
    }
}

fn prepare_tmp_root_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        clean_dir(path, true)
             .map_err(|x| format!("Error removing directory: {}", x))?;
    }
    try_msg!(Dir::new(path).recursive(true).create(),
        "Error creating directory: {err}");
    let rootdir = path.join("root");
    try_msg!(Dir::new(&rootdir).create(),
        "Error creating directory: {err}");

    let tgtbase = Path::new("/vagga/container");
    try_msg!(Dir::new(&tgtbase).create(),
        "Error creating directory: {err}");
    try_msg!(BindMount::new(path, &tgtbase).mount(),
        "mount container: {err}");

    let tgtroot = Path::new("/vagga/root");
    try_msg!(Dir::new(&tgtroot).create(),
        "Error creating directory: {err}");
    try_msg!(BindMount::new(&rootdir, &tgtroot).mount(),
        "mount container root: {err}");

    try_msg!(Dir::new(&tgtroot.join("dev")).create(),
        "Error creating directory: {err}");
    try_msg!(Dir::new(&tgtroot.join("sys")).create(),
        "Error creating directory: {err}");
    try_msg!(Dir::new(&tgtroot.join("proc")).create(),
        "Error creating directory: {err}");
    try_msg!(Dir::new(&tgtroot.join("run")).create(),
        "Error creating directory: {err}");
    try_msg!(Dir::new(&tgtroot.join("tmp")).mode(0o1777).create(),
        "Error creating directory: {err}");
    try_msg!(Dir::new(&tgtroot.join("work")).create(),
        "Error creating directory: {err}");
    return Ok(());
}

pub fn unmount_service_dirs() -> Result<(), String> {
    unmount(&Path::new("/vagga/root"))?;
    remove_dir(&Path::new("/vagga/root"))
        .map_err(|e| format!("Can't unlink root: {}", e))?;
    unmount(&Path::new("/vagga/container"))?;
    remove_dir(&Path::new("/vagga/container"))
        .map_err(|e| format!("Can't unlink root: {}", e))?;
    Ok(())
}

pub fn commit_root(tmp_path: &Path, final_path: &Path) -> Result<(), String> {
    let mut path_to_remove = None;
    if final_path.exists() {
        let rempath = tmp_path.with_file_name(
            // TODO(tailhook) consider these unwraps
            tmp_path.file_name().unwrap().to_str()
            .unwrap().to_string() + ".old");
        if rempath.is_dir() {
            clean_dir(&rempath, true)
                 .map_err(|x| format!("Error removing old dir: {}", x))?;
        }
        rename(final_path, &rempath)
             .map_err(|x| format!("Error renaming old dir: {}", x))?;
        path_to_remove = Some(rempath);
    }
    File::create(tmp_path.join("last_use"))
        .map_err(|e| format!("Can't write image usage info: {}", e))?;
    rename(tmp_path, final_path)
         .map_err(|x| format!("Error renaming dir: {}", x))?;
    if let Some(ref path_to_remove) = path_to_remove {
        clean_dir(path_to_remove, true)
             .map_err(|x| format!("Error removing old dir: {}", x))?;
    }
    return Ok(());
}

pub fn get_version_hash(container: &str, wrapper: &Wrapper)
    -> Result<Option<String>, String>
{
    _get_version_hash(&Options::for_container(container), wrapper)
}

fn _get_version_hash(options: &Options, wrapper: &Wrapper)
    -> Result<Option<String>, String>
{
    let mut cmd = Command::new("/vagga/bin/vagga");
    cmd.arg("__version__");
    cmd.gid(0);
    cmd.groups(Vec::new());
    cmd.arg(&options.container);
    cmd.arg("--settings");
    cmd.arg(serde_json::to_string(&**wrapper.settings).unwrap());
    if options.debug_versioning {
        cmd.arg("--debug-versioning");
    }
    if options.dump_version_data {
        cmd.arg("--dump-version-data");
    }
    cmd.env_clear();
    copy_env_vars(&mut cmd, &wrapper.settings);
    if let Ok(x) = env::var("RUST_LOG") {
        cmd.env("RUST_LOG", x);
    }
    if let Ok(x) = env::var("RUST_BACKTRACE") {
        cmd.env("RUST_BACKTRACE", x);
    }
    if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
        cmd.env("VAGGA_DEBUG_CMDENV", x);
    }
    match capture_fd3_status(cmd) {
        Ok((ExitStatus::Exited(0), val)) => {
            String::from_utf8(val)
                .map_err(|e| format!("Can't decode version: {}", e))
                .map(Some)
        },
        Ok((ExitStatus::Exited(29), _)) => Ok(None),
        Ok((status, _)) => return Err(format!("Versioner exited {}", status)),
        Err(e) => return Err(format!("Could not run versioner: {}", e)),
    }
}

fn build_container(cont_info: &ContainerInfo, wrapper: &Wrapper)
    -> Result<String, String>
{
    let dir_name = _build_container(&cont_info, wrapper)?;
    let destlink = Path::new("/work/.vagga").join(cont_info.name);
    let tmplink = destlink.with_extension("tmp");
    if tmplink.exists() {
        remove_file(&tmplink)
            .map_err(|e| format!("Error removing temporary link: {}", e))?;
    }
    let roots = if wrapper.ext_settings.storage_dir.is_some() {
        // at this point `.lnk` should already be created even if we use
        // storage-subdir-from-env-var
        Path::new(".lnk/.roots")
    } else {
        Path::new(".roots")
    };
    let linkval = roots.join(&dir_name).join("root");
    if cont_info.container.auto_clean && !wrapper.settings.disable_auto_clean {
        match read_link(&destlink) {
            Ok(ref oldval) if oldval != &linkval => {
                let oldname = oldval.iter().rev().nth(1)
                    .ok_or(format!("Bad link {:?} -> {:?}",
                        destlink, oldval))?;
                let base = Path::new("/vagga/base/.roots");
                let dir = base.join(&oldname);
                let tmpdir = base.join({
                    let mut tmpname = OsString::from(".tmp");
                    tmpname.push(oldname);
                    tmpname
                });
                rename(&dir, &tmpdir)
                    .map_err(|e| error!("Error renaming old dir: {}", e)).ok();
                clean_dir(&tmpdir, true)
                    .map_err(|e| error!("Error removing old dir: {}", e)).ok();
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == NotFound => {}
            Err(e) => {
                return Err(format!("Error reading symlink {:?}: {}", destlink, e));
            }
        };
    }
    symlink(&linkval, &tmplink)
         .map_err(|e| format!("Error symlinking container: {}", e))?;
    rename(&tmplink, &destlink)
         .map_err(|e| format!("Error renaming symlink: {}", e))?;
    return Ok(dir_name);
}

fn compare_files<A: AsRef<Path>, B: AsRef<Path>>(a: A, b: B)
    -> io::Result<bool>
{
    let mut abuf = Vec::with_capacity(1024);
    let mut bbuf = Vec::with_capacity(1024);
    File::open(a.as_ref()).and_then(|mut f| f.read_to_end(&mut abuf))?;
    File::open(b.as_ref()).and_then(|mut f| f.read_to_end(&mut bbuf))?;
    Ok(abuf != bbuf)
}

fn uidmap_differs(container_path: &Path) -> bool {
    compare_files(
        "/proc/self/uid_map",
        container_path.join("uid_map")
    ).unwrap_or(true) ||
    compare_files(
        "/proc/self/uid_map",
        container_path.join("uid_map")
    ).unwrap_or(true)
}

fn _check_exists(container: &str, force: bool, wrapper: &Wrapper,
    version: &mut Option<String>)
    -> Result<Option<String>, String>
{
    if version.is_none() {
        *version = get_version_hash(container, wrapper)?.and_then(|ver| {
            if ver.len() != 128 || !ver[..].is_ascii() {
                None
            } else {
                Some(ver)
            }
        });
    }
    let ver = if let &mut Some(ref ver) = version {
        ver
    } else {
        return Ok(None);
    };
    let dir_name = format!("{}.{}", container, &ver[..8]);
    let finalpath = Path::new("/vagga/base/.roots")
        .join(&dir_name);
    debug!("Container path: {:?} (force: {}) {}",
        finalpath, force, finalpath.exists());
    if finalpath.exists() && !force {
        if uidmap_differs(&finalpath) {
            warn!("Current uidmap differs from uidmap of container \
            when it was built.  This probably means that you \
            either running vagga wrong user id or changed uid or \
            subuids of your user since container was built. We'll \
            rebuilt container to make sure it has proper \
            permissions");
        } else {
            debug!("Path {:?} is already built", finalpath);
            return Ok(Some(dir_name));
        }
    }
    return Ok(None);
}

fn _build_container(cont_info: &ContainerInfo, wrapper: &Wrapper)
    -> Result<String, String>
{
    let mut ver = None;
    if let Some(dir_name) = _check_exists(
        cont_info.name, cont_info.force, wrapper, &mut ver)?
    {
        return Ok(dir_name);
    }
    debug!("Container version: {:?}", ver);

    let lock_name = cont_info.tmp_root_dir.with_file_name(
        format!(".tmp.{}.lock", cont_info.name));
    let mut _lock_guard = if wrapper.settings.build_lock_wait {
        Lock::exclusive_wait(&lock_name,
            "Other process is doing a build. Waiting...")
        .map_err(|e| format!("Can't lock container build ({}). \
                              Aborting...", e))?

    } else {
        Lock::exclusive(&lock_name)
        .map_err(|e| format!("Can't lock container build ({}). \
            Probably other process is doing build. Aborting...", e))?
    };

    // must recheck after getting lock to avoid race condition
    if let Some(dir_name) = _check_exists(
        cont_info.name, cont_info.force, wrapper, &mut ver)?
    {
        return Ok(dir_name);
    }

    prepare_tmp_root_dir(&cont_info.tmp_root_dir).map_err(|e|
        format!("Error preparing root dir: {}", e))?;

    let build_start = Instant::now();

    match maybe_build_from_image(cont_info, &ver, wrapper) {
        Ok(true) => {}
        Ok(false) => {
            let mut cmd = Command::new("/vagga/bin/vagga");
            cmd.arg("__builder__");
            cmd.gid(0);
            cmd.groups(Vec::new());
            cmd.unshare(
                [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
            cmd.arg(cont_info.name);
            if let Some(ref ver) = ver {
                cmd.arg("--container-version");
                cmd.arg(format!("{}.{}", cont_info.name, &ver[..8]));
            }
            cmd.arg("--settings");
            cmd.arg(serde_json::to_string(&**wrapper.settings).unwrap());
            cmd.env_clear();
            copy_env_vars(&mut cmd, &wrapper.settings);
            if let Ok(x) = env::var("RUST_LOG") {
                cmd.env("RUST_LOG", x);
            }
            if let Ok(x) = env::var("RUST_BACKTRACE") {
                cmd.env("RUST_BACKTRACE", x);
            }
            if let Ok(x) = env::var("VAGGA_DEBUG_CMDENV") {
                cmd.env("VAGGA_DEBUG_CMDENV", x);
            }

            match cmd.status() {
                Ok(s) if s.success() => {}
                Ok(s) => return Err(format!("Builder {}", s)),
                Err(e) => return Err(format!("Error running builder: {}", e)),
            };
        },
        Err(e) => {
            return Err(format!("Error when building from image: {}", e));
        },
    }

    unmount_service_dirs()?;

    let ver = if let Some(ver) = ver { ver }
        else {
            match get_version_hash(cont_info.name, wrapper) {
                Ok(Some(ver)) => {
                    if ver.len() == 128 && ver[..].is_ascii() {
                        ver
                    } else {
                        return Err(format!("Internal Error: \
                                Wrong version returned: {:?}", ver));
                    }
                }
                Ok(None) => {
                    return Err(format!("Internal Error: \
                            Can't version even after build"));
                },
                Err(e) => return Err(e),
            }
        };
    let dir_name = format!("{}.{}", cont_info.name, &ver[..8]);
    let roots_dir = PathBuf::from("/vagga/base/.roots");
    let finalpath = roots_dir.join(&dir_name);

    if wrapper.settings.index_all_images &&
        wrapper.settings.hard_link_identical_files
    {
        find_and_hardlink_identical_files(
            &wrapper, cont_info, &roots_dir, &finalpath)?;
    }

    debug!("Committing {:?} -> {:?}", cont_info.tmp_root_dir, &finalpath);
    match commit_root(&cont_info.tmp_root_dir, &finalpath) {
        Ok(()) => {}
        Err(x) => {
            return Err(format!("Error committing root dir: {}", x));
        }
    }
    let duration = build_start.elapsed();
    warn!("Container {} ({}) built in {} seconds.",
        cont_info.name, &ver[..8], duration.as_secs());
    return Ok(dir_name);
}

fn maybe_build_from_image(cont_info: &ContainerInfo, version: &Option<String>,
    wrapper: &Wrapper)
    -> Result<bool, String>
{
    if cont_info.force || cont_info.no_image {
        return Ok(false);
    }
    if let Some(ref image_url_tmpl) = cont_info.container.image_cache_url {
        if let Some(ref version) = *version {
            let image_url = image_url_tmpl
                .replace("${container_name}", cont_info.name)
                .replace("${short_hash}", &version[..8]);
            match _build_from_image(cont_info.name, cont_info.container,
                &wrapper.config, &wrapper.settings, &image_url)
            {
                Ok(()) => {
                    Ok(true)
                },
                Err(e) => {
                    error!("Error when unpacking image: {}. \
                            Will clean and build it locally...", e);
                    unmount_service_dirs().map_err(|e|
                        format!("Error cleaning service dirs: {}", e))?;
                    prepare_tmp_root_dir(&cont_info.tmp_root_dir).map_err(|e|
                        format!("Error preparing root dir: {}", e))?;
                    Ok(false)
                }
            }
        } else {
            debug!("Cannot build container from image. \
                    Container's version will be available only after build.");
            Ok(false)
        }
    } else {
        Ok(false)
    }
}

fn _build_from_image(name: &str, container: &Container,
    config: &Config, settings: &Settings, image_cache_url: &String)
    -> Result<(), String>
{
    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let mut ctx = Context::new(config, name.to_string(),
                               container, settings.clone());

    let (filename, downloaded) = maybe_download_and_check_hashsum(
        &mut ctx.capsule, image_cache_url, None, false)?;
    warn!("Unpacking image...");
    let cont_dir = Path::new("/vagga/container");
    let root_dir = Path::new("/vagga/root");
    match unpack_file(&mut ctx, &filename, root_dir, &[], &[], true) {
        Ok(_) => {
            info!("Succesfully unpack image {}", image_cache_url);
            // If container is okay, we need to store uid_map used for
            // unpacking
            copy("/proc/self/uid_map", "/vagga/container/uid_map")
                .map_err(|e| format!("Error copying uid_map: {}", e))?;
            copy("/proc/self/gid_map", "/vagga/container/gid_map")
                .map_err(|e| format!("Error copying gid_map: {}", e))?;
            // Remove image from local cache after unpacking
            if downloaded {
                remove_file(&filename)
                    .map_err(|e| error!(
                        "Error unlinking cache file: {}", e)).ok();
            }
            if settings.index_all_images {
                warn!("Indexing container...");
                write_container_signature(cont_dir)?;
            }
        },
        Err(e) => {
            return Err(format!("Error unpacking image {}: {}",
                image_cache_url, e));
        },
    }

    Ok(())
}

pub fn build_container_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut name: String = "".to_string();
    let mut force: bool = false;
    let mut no_image: bool = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Internal vagga tool to setup basic system sandbox
            ");
        ap.refer(&mut name)
            .add_argument("container_name", Store,
                "Container name to build");
        ap.refer(&mut force)
            .add_option(&["--force"], StoreTrue,
                "Force build even if container is considered up to date");
        ap.refer(&mut no_image)
            .add_option(&["--no-image-download"], StoreTrue,
                "Do not download image");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;

    build_wrapper(&name, force, no_image, wrapper)
    .map(|x| unsafe { File::from_raw_fd(3) }.write_all(x.as_bytes()).unwrap())
    .map(|_| 0)
}

pub fn build_wrapper(name: &str, force: bool, no_image: bool, wrapper: &Wrapper)
    -> Result<String, String>
{
    let container = wrapper.config.containers.get(name)
        .ok_or(format!("Container {:?} not found", name))?;
    for &Step(ref step) in container.setup.iter() {
        if let Some(name) = step.is_dependent_on() {
            build_wrapper(name, force, no_image, wrapper)
                .map(|x| debug!("Built container with name {}", x))
                .map(|()| 0)?;
        }
    }

    let cont_info = ContainerInfo::new(name, container, force, no_image);
    return build_container(&cont_info, wrapper)
}

pub fn print_version_hash_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let opt = match Options::parse(&cmdline, true) {
        Ok(x) => x,
        Err(e) => return Ok(e),
    };
    setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)?;
    if let Some(hash) = _get_version_hash(&opt, wrapper)? {
        let res = if opt.short { &hash[..8] } else { &hash[..] };
        if opt.fd3 {
            unsafe { File::from_raw_fd(3) }.write_all(res.as_bytes())
                .map_err(|e| format!("Error writing to fd 3: {}", e))?;
        } else {
            println!("{}", res);
        }
        Ok(0)
    } else {
        Ok(29)
    }
}

fn find_and_hardlink_identical_files(wrapper: &Wrapper,
    cont_info: &ContainerInfo, roots_dir: &Path, finalpath: &Path)
    -> Result<(), String>
{
    let (tmp_root_dir, project_name, _cont_dirs) =
        if wrapper.settings.hard_link_between_projects &&
            wrapper.ext_settings.storage_dir.is_some()
    {
        let storage_dir = Path::new("/vagga/storage");
        let project_path = try_msg!(
            read_link(Path::new("/work/.vagga/.lnk")),
            "Cannot read .vagga/.lnk symlink: {err}");
        let project_name = project_path.file_name()
            .ok_or(format!("Cannot detect project name"))?
            .to_str()
            .ok_or(format!("Cannot convert project name to string"))?
            .to_string();
        let tmp_dir_name = cont_info.tmp_root_dir.file_name()
            .ok_or(format!("Cannot detect tmp dir name"))?;
        let tmp_root_dir = storage_dir
            .join(&project_name)
            .join(".roots")
            .join(tmp_dir_name);
        let cont_dirs = collect_containers_from_storage(storage_dir)?;
        (tmp_root_dir, Some(project_name), cont_dirs)
    } else {
        let cont_dirs = collect_container_dirs(&roots_dir, None)?;
        (cont_info.tmp_root_dir.clone(), None, cont_dirs)
    };
    // Collect only containers with signature file
    let mut cont_dirs = _cont_dirs.iter()
        .filter(|d| d.path != finalpath)
        .filter(|d| d.path.join("index.ds1").is_file())
        .map(|d| d)
        .collect::<Vec<_>>();
    // Sort by project, container name and date modified
    cont_dirs.sort_by_key(|d| {
        (d.project == project_name,
         &d.project,
         d.name == cont_info.name,
         &d.name,
         d.modified)
    });
    // Group by project and container name
    let grouped_cont_dirs = cont_dirs.into_iter()
        .rev()
        .group_by(|d| (&d.project, &d.name));
    // Take only 3 last version of each container
    let cont_dirs = grouped_cont_dirs.into_iter()
        .flat_map(|(_, group)| group.take(3));
    let cont_paths = cont_dirs.into_iter()
        .map(|d| &d.path);
    match hardlink_container_files(&tmp_root_dir, cont_paths)
    {
        Ok((count, size)) if count > 0 => warn!(
            "Found and linked {} ({}) identical files \
             from other containers", count, human_size(size)),
        Ok(_) => {
            debug!("No hardlinks done: either no source directories found \
                    or no identical files");
        },
        Err(msg) => warn!("Error when linking container files: {}", msg),
    }
    Ok(())
}
