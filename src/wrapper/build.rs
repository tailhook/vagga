use std::env;
use std::fs::{File, read_link};
use std::io::Write;
use std::ffi::OsString;
use std::io::ErrorKind::NotFound;
use std::ascii::AsciiExt;
use std::fs::{rename};
use std::fs::{remove_file, remove_dir};
use std::io::{stdout, stderr};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::os::unix::io::FromRawFd;

use argparse::{ArgumentParser, Store, StoreTrue};
use rustc_serialize::json;
use unshare::{Command, Namespace, ExitStatus};
use libmount::BindMount;

use container::util::clean_dir;
use container::mount::{unmount};
use container::uidmap::{map_users};
use config::{Container, Step};
use config::builders::Builder as B;
use config::builders::Source as S;
use file_util::{create_dir, Lock};
use process_util::{capture_fd3_status, set_uidmap, copy_env_vars};
use super::Wrapper;
use super::setup;


pub fn prepare_tmp_root_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        try!(clean_dir(path, true)
             .map_err(|x| format!("Error removing directory: {}", x)));
    }
    try_msg!(create_dir(path, true),
         "Error creating directory: {err}");
    let rootdir = path.join("root");
    try_msg!(create_dir(&rootdir, false),
         "Error creating directory: {err}");

    let tgtbase = Path::new("/vagga/container");
    try_msg!(create_dir(&tgtbase, false),
         "Error creating directory: {err}");
    try_msg!(BindMount::new(path, &tgtbase).mount(),
        "mount container: {err}");

    let tgtroot = Path::new("/vagga/root");
    try_msg!(create_dir(&tgtroot, false),
         "Error creating directory: {err}");
    try_msg!(BindMount::new(&rootdir, &tgtroot).mount(),
        "mount container root: {err}");

    try_msg!(create_dir(&tgtroot.join("dev"), false),
         "Error creating directory: {err}");
    try_msg!(create_dir(&tgtroot.join("sys"), false),
         "Error creating directory: {err}");
    try_msg!(create_dir(&tgtroot.join("proc"), false),
         "Error creating directory: {err}");
    try_msg!(create_dir(&tgtroot.join("work"), false),
         "Error creating directory: {err}");
    return Ok(());
}

pub fn commit_root(tmp_path: &Path, final_path: &Path) -> Result<(), String> {
    let mut path_to_remove = None;
    if final_path.exists() {
        let rempath = tmp_path.with_file_name(
            // TODO(tailhook) consider these unwraps
            tmp_path.file_name().unwrap().to_str()
            .unwrap().to_string() + ".old");
        if rempath.is_dir() {
            try!(clean_dir(&rempath, true)
                 .map_err(|x| format!("Error removing old dir: {}", x)));
        }
        try!(rename(final_path, &rempath)
             .map_err(|x| format!("Error renaming old dir: {}", x)));
        path_to_remove = Some(rempath);
    }
    try!(rename(tmp_path, final_path)
         .map_err(|x| format!("Error renaming dir: {}", x)));
    if let Some(ref path_to_remove) = path_to_remove {
        try!(clean_dir(path_to_remove, true)
             .map_err(|x| format!("Error removing old dir: {}", x)));
    }
    return Ok(());
}

pub fn get_version_hash(container: &String, wrapper: &Wrapper)
    -> Result<Option<String>, String>
{
    let cconfig = try!(wrapper.config.containers.get(container)
        .ok_or(format!("Container {} not found", container)));
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));

    let mut cmd = Command::new("/vagga/bin/vagga");
    cmd.arg0("vagga_version");
    set_uidmap(&mut cmd, &uid_map, false);
    cmd.arg(&container);
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

pub fn build_container(container: &String, force: bool, no_image: bool,
    wrapper: &Wrapper)
    -> Result<String, String>
{
    let cconfig = try!(wrapper.config.containers.get(container)
        .ok_or(format!("Container {} not found", container)));
    let name = try!(_build_container(cconfig, container, force, no_image, wrapper));
    let destlink = Path::new("/work/.vagga").join(&container);
    let tmplink = destlink.with_extension("tmp");
    if tmplink.exists() {
        try!(remove_file(&tmplink)
            .map_err(|e| format!("Error removing temporary link: {}", e)));
    }
    let roots = if wrapper.ext_settings.storage_dir.is_some() {
        Path::new(".lnk/.roots")
    } else {
        Path::new(".roots")
    };
    let linkval = roots.join(&name).join("root");
    if cconfig.auto_clean {
        match read_link(&destlink) {
            Ok(ref oldval) if oldval != &linkval => {
                let oldname = try!(oldval.iter().rev().nth(1)
                    .ok_or(format!("Bad link {:?} -> {:?}",
                        destlink, oldval)));
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
                return Err(format!("Error reading symlin {:?}: {}",
                    destlink, e));
            }
        };
    }
    try!(symlink(&linkval, &tmplink)
         .map_err(|e| format!("Error symlinking container: {}", e)));
    try!(rename(&tmplink, &destlink)
         .map_err(|e| format!("Error renaming symlink: {}", e)));
    return Ok(name);
}

pub fn _build_container(cconfig: &Container, container: &String,
    force: bool, no_image: bool, wrapper: &Wrapper)
    -> Result<String, String>
{
    let uid_map = try!(map_users(wrapper.settings,
        &cconfig.uids, &cconfig.gids));

    let ver = match get_version_hash(container, wrapper) {
        Ok(Some(ver)) => {
            if ver.len() == 64 && ver[..].is_ascii() {
                let name = format!("{}.{}", container, &ver[..8]);
                let finalpath = Path::new("/vagga/base/.roots")
                    .join(&name);
                debug!("Container path: {:?} (force: {}) {}", finalpath, force,
                    finalpath.exists());
                if finalpath.exists() && !force {
                    debug!("Path {} is already built",
                           finalpath.display());
                    return Ok(name);
                }
                Some(ver)
            } else {
                error!("Wrong version hash: {:?}", ver);
                None
            }
        }
        Ok(None) => None,
        Err(e) => return Err(e),
    };
    debug!("Container version: {:?}", ver);
    let tmppath = PathBuf::from(
        &format!("/vagga/base/.roots/.tmp.{}", container));

    let _lock_guard = try!(Lock::exclusive(
            &tmppath.with_file_name(format!(".tmp.{}.lock", container)))
        .map_err(|e| format!("Can't lock container build ({}). \
            Probably other process is doing build. Aborting...", e)));

    match prepare_tmp_root_dir(&tmppath) {
        Ok(()) => {}
        Err(x) => {
            return Err(format!("Error preparing root dir: {}", x));
        }
    }

    let mut cmd = Command::new("/vagga/bin/vagga");
    cmd.arg0("vagga_build");
    set_uidmap(&mut cmd, &uid_map, false);
    cmd.unshare(
        [Namespace::Mount, Namespace::Ipc, Namespace::Pid].iter().cloned());
    cmd.arg(&container);
    if let Some(ref ver) = ver {
        cmd.arg("--container-version");
        cmd.arg(format!("{}.{}", container, &ver[..8]));
    }
    if force || no_image {
        cmd.arg("--no-image-download");
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

    let result = cmd.status();
    try!(unmount(&Path::new("/vagga/root")));
    try!(remove_dir(&Path::new("/vagga/root"))
        .map_err(|e| format!("Can't unlink root: {}", e)));
    try!(unmount(&Path::new("/vagga/container")));
    try!(remove_dir(&Path::new("/vagga/container"))
        .map_err(|e| format!("Can't unlink root: {}", e)));
    match result {
        Ok(s) if s.success() => {}
        Ok(s) => return Err(format!("Builder {}", s)),
        Err(e) => return Err(format!("Error running builder: {}", e)),
    };

    let ver = if let Some(ver) = ver { ver }
        else {
            match get_version_hash(container, wrapper) {
                Ok(Some(ver)) => {
                    if ver.len() == 64 && ver[..].is_ascii() {
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
    let name = format!("{}.{}", container,
        &ver[..8]);
    let finalpath = Path::new("/vagga/base/.roots").join(&name);
    debug!("Committing {} -> {}", tmppath.display(),
                                  finalpath.display());
    match commit_root(&tmppath, &finalpath) {
        Ok(()) => {}
        Err(x) => {
            return Err(format!("Error committing root dir: {}", x));
        }
    }
    return Ok(name);
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
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));

    build_wrapper(&name, force, no_image, wrapper)
    .map(|x| unsafe { File::from_raw_fd(3) }.write_all(x.as_bytes()).unwrap())
    .map(|_| 0)
}

pub fn build_wrapper(name: &String, force: bool, no_image: bool, wrapper: &Wrapper)
    -> Result<String, String>
{
    let container = try!(wrapper.config.containers.get(name)
        .ok_or(format!("Container {:?} not found", name)));
    for &Step(ref step) in container.setup.iter() {
        match step {
            &B::Container(ref name) => {
                try!(build_wrapper(name, force, no_image, wrapper)
                    .map(|x| debug!("Built container with name {}", x))
                    .map(|()| 0));
            }
            &B::Build(ref binfo) => {
                try!(build_wrapper(&binfo.container, force, no_image, wrapper)
                    .map(|x| debug!("Built container with name {}", x))
                    .map(|()| 0));
            }
            &B::SubConfig(ref cfg) => {
                match cfg.source {
                    S::Directory => {}
                    S::Container(ref name) => {
                        try!(build_wrapper(name, force, no_image, wrapper)
                            .map(|x| debug!("Built container with name {}", x))
                            .map(|()| 0));
                    }
                    S::Git(ref _git) => {
                        unimplemented!();
                    }
                }
            }
            _ => {}
        }
    }

    return build_container(name, force, no_image, wrapper)
}

pub fn print_version_hash_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut name: String = "".to_string();
    let mut short = false;
    let mut fd3 = false;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Prints version hash of the container without building it. If
            this command exits with code 29, then container can't be versioned
            before the build.
            ");
        ap.refer(&mut name)
            .add_argument("container_name", Store,
                "Container name to build");
        ap.refer(&mut short)
            .add_option(&["-s", "--short"], StoreTrue,
                "Print short container version, like used in directory names \
                 (8 chars)");
        ap.refer(&mut fd3)
            .add_option(&["--fd3"], StoreTrue,
                "Print into file descriptor #3 instead of stdout");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
    try!(setup::setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings));
    if let Some(hash) = try!(get_version_hash(&name, wrapper)) {
        let res = if short { &hash[..8] } else { &hash[..] };
        if fd3 {
            try!(unsafe { File::from_raw_fd(3) }.write_all(res.as_bytes())
                .map_err(|e| format!("Error writing to fd 3: {}", e)));
        } else {
            println!("{}", res);
        }
        Ok(0)
    } else {
        Ok(29)
    }
}

