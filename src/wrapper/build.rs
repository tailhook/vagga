use std::env;
use std::fs::File;
use std::fs::{hard_link, read_link, read_dir, rename, remove_dir, remove_file};
use std::ffi::OsString;
use std::ascii::AsciiExt;
use std::io::{self, Read, Write, BufReader};
use std::io::{stdout, stderr};
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};
use std::os::unix::fs::{MetadataExt, symlink};
use std::os::unix::io::FromRawFd;

use argparse::{ArgumentParser, Store, StoreTrue};
use rustc_serialize::json;
use unshare::{Command, Namespace, ExitStatus};
use libmount::BindMount;
use dir_signature::v1::{Entry, EntryKind, Parser};
use dir_signature::v1::merge::FileMergeBuilder;
use itertools::Itertools;

use builder::context::Context;
use builder::commands::tarcmd::unpack_file;
use builder::guard;
use capsule::download::maybe_download_and_check_hashsum;
use config::{Config, Container, Settings};
use container::util::clean_dir;
use container::mount::{unmount};
use file_util::{Dir, Lock, copy};
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

pub fn prepare_tmp_root_dir(path: &Path) -> Result<(), String> {
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
    cmd.arg(json::encode(wrapper.settings).unwrap());
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
            cmd.arg(json::encode(wrapper.settings).unwrap());
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
    let finalpath = Path::new("/vagga/base/.roots").join(&dir_name);

    if wrapper.settings.index_all_images &&
        wrapper.settings.hard_link_identical_files
    {
        match find_and_link_identical_files(
            cont_info.name, &cont_info.tmp_root_dir, &finalpath)
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
        &mut ctx.capsule, image_cache_url, None)?;
    warn!("Unpacking image...");
    match unpack_file(&mut ctx, &filename, &Path::new("/vagga/root"), &[], &[], true) {
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
                guard::index_image()?;
            }
        },
        Err(e) => {
            return Err(format!("Error unpacking image {}: {}",
                image_cache_url, e));
        },
    }

    Ok(())
}

#[cfg(feature="containers")]
fn find_and_link_identical_files(container_name: &str,
    tmp_dir: &Path, final_dir: &Path)
    -> Result<(u32, u64), String>
{
    let container_root = tmp_dir.join("root");
    let main_ds_path = tmp_dir.join("index.ds1");
    if !main_ds_path.exists() {
        warn!("No index file exists. Can't hardlink");
        return Ok((0, 0));
    }
    let main_ds_reader = BufReader::new(try_msg!(File::open(&main_ds_path),
        "Error opening file {path:?}: {err}", path=&main_ds_path));
    let mut main_ds_parser = try_msg!(Parser::new(main_ds_reader),
        "Error parsing signature file: {err}");

    let _paths_names_times = get_container_paths_names_times(final_dir)?;
    let mut paths_names_times = _paths_names_times.iter()
        .map(|&(ref p, ref n, ref t)| (p, n, t))
        .collect::<Vec<_>>();
    // Sort by current container name equality
    // then by container name and then by modification date
    paths_names_times.sort_by_key(|&(_, n, t)| {
        (n == container_name, n, t)
    });
    let mut merged_ds_builder = FileMergeBuilder::new();
    for (_, cont_group) in paths_names_times
        .into_iter()
        .rev()
        .group_by(|&(_, n, _)| n)
        .into_iter()
    {
        for (cont_path, _, _) in cont_group.take(5) {
            merged_ds_builder.add(&cont_path.join("root"),
                                  &cont_path.join("index.ds1"));
        }
    }
    let mut merged_ds = try_msg!(merged_ds_builder.finalize(),
        "Error parsing signature files: {err}");
    let mut merged_ds_iter = merged_ds.iter();

    let tmp = tmp_dir.join(".link.tmp");
    let mut count = 0;
    let mut size = 0;
    for entry in main_ds_parser.iter() {
        match entry {
            Ok(Entry::File{
                path: ref lnk_path,
                exe: lnk_exe,
                size: lnk_size,
                hashes: ref lnk_hashes,
            }) => {
                let lnk = container_root.join(
                    match lnk_path.strip_prefix("/") {
                        Ok(lnk_path) => lnk_path,
                        Err(_) => continue,
                    });
                let lnk_stat = lnk.symlink_metadata().map_err(|e|
                    format!("Error querying file stats: {}", e))?;
                for tgt_entry in merged_ds_iter
                    .advance(&EntryKind::File(lnk_path))
                {
                    match tgt_entry {
                        (tgt_base_path,
                         Ok(Entry::File{
                             path: ref tgt_path,
                             exe: tgt_exe,
                             size: tgt_size,
                             hashes: ref tgt_hashes}))
                            if lnk_exe == tgt_exe &&
                            lnk_size == tgt_size &&
                            lnk_hashes == tgt_hashes =>
                        {
                            let tgt = tgt_base_path.join(
                                match tgt_path.strip_prefix("/") {
                                    Ok(path) => path,
                                    Err(_) => continue,
                                });
                            let tgt_stat = tgt.symlink_metadata().map_err(|e|
                                format!("Error querying file stats: {}", e))?;
                            if lnk_stat.mode() != tgt_stat.mode() ||
                                lnk_stat.uid() != tgt_stat.uid() ||
                                lnk_stat.gid() != lnk_stat.gid()
                            {
                                continue;
                            }
                            if let Err(_) = hard_link(&tgt, &tmp) {
                                remove_file(&tmp).map_err(|e|
                                    format!("Error removing file after failed \
                                             hard linking: {}", e))?;
                                continue;
                            }
                            if let Err(_) = rename(&tmp, &lnk) {
                                remove_file(&tmp).map_err(|e|
                                    format!("Error removing file after failed \
                                             renaming: {}", e))?;
                                continue;
                            }
                            count += 1;
                            size += tgt_size;
                            break;
                        },
                        _ => continue,
                    }
                }
            },
            _ => {},
        }
    }

    Ok((count, size))
}

#[cfg(not(feature="containers"))]
fn find_and_link_identical_files(container_name: &str,
    tmp_dir: &Path, final_dir: &Path)
    -> Result<(u32, u64), String>
{
    unimplemented!();
}

fn get_container_paths_names_times(exclude_path: &Path)
    -> Result<Vec<(PathBuf, String, SystemTime)>, String>
{
    Ok(try_msg!(read_dir("/vagga/base/.roots"),
                "Error reading directory: {err}")
        .filter_map(|x| x.ok())
        .map(|x| x.path())
        .filter(|p| {
            p != exclude_path &&
                p.is_dir() &&
                p.join("index.ds1").is_file()
        })
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string())
                .map(|n| (p, n))
        })
        .filter(|&(_, ref d)| !d.starts_with("."))
        .filter_map(|(p, d)| {
            let mut dir_name_parts = d.rsplitn(2, '.');
            dir_name_parts.next();
            dir_name_parts.next()
                .map(|n| (p, n.to_string()))
        })
        .filter_map(|(p, n)| {
            p.metadata()
                .and_then(|m| m.modified()).ok()
                .map(|t| (p, n, t))
        })
        .collect::<Vec<_>>())
}

fn human_size(size: u64) -> String {
    fn format_size(s: f64, p: &str) -> String {
        if s < 10.0 {
            format!("{:.1}{}B", s, p)
        } else {
            format!("{:.0}{}B", s, p)
        }
    }

    let mut s = size as f64;
    for prefix in &["", "K", "M", "G", "T"][..] {
        if s < 1000.0 {
            return format_size(s, prefix);
        }
        s /= 1000.0;
    }
    return format_size(s, "P");
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

