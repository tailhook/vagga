use std::io::{self, Read, Write};
use std::fs::{create_dir, File};

use unshare::{Command};
use libmount::Overlay;

use crate::process_util::convert_status;
use crate::wrapper::capsule;

use super::Wrapper;
use super::setup::setup_base_filesystem;


pub fn run_interactive_build_shell(wrapper: &Wrapper) -> i32 {
    if let Err(text) = setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)
    {
        error!("Error setting base file system: {}", &text);
        return 122;
    }

    // Make symlinks to make interactive life easier
    capsule::symlink_busybox_commands()
        .map_err(|e| error!("{}", e)).ok();

    match Command::new("/vagga/bin/busybox")
            .arg("sh")
            .env("PATH", "/vagga/bin:/bin")
        .status()
        .map_err(|e| format!("Can't run busybox: {}", e))
    {
        Ok(x) => convert_status(x),
        Err(x) => {
            error!("Error running build_shell: {}", x);
            return 127;
        }
    }
}

fn create_dirs() -> io::Result<()> {
    create_dir("/tmp")?;
    create_dir("/tmp/dir1")?;
    File::create("/tmp/dir1/f1.txt")
        .and_then(|mut f| f.write_all(b"one"))?;
    create_dir("/tmp/dir2")?;
    File::create("/tmp/dir2/f2.txt")
        .and_then(|mut f| f.write_all(b"two"))?;
    create_dir("/tmp/dir3")?;
    File::create("/tmp/dir3/f3.txt")
        .and_then(|mut f| f.write_all(b"three"))?;
    create_dir("/tmp/wrk")?;
    create_dir("/tmp/merged")?;
    Ok(())
}

fn err(e: &'static str) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Other, e))
}

fn check_read_write() -> io::Result<()> {
    let mut buf = String::with_capacity(100);
    File::open("/tmp/merged/f1.txt")
        .and_then(|mut f| f.read_to_string(&mut buf))?;
    if buf != "one" {
        return err("f1.txt has wrong data");
    }
    buf.clear();
    File::open("/tmp/merged/f2.txt")
        .and_then(|mut f| f.read_to_string(&mut buf))?;
    if buf != "two" {
        return err("f2.txt has wrong data");
    }
    buf.clear();
    File::open("/tmp/merged/f3.txt")
        .and_then(|mut f| f.read_to_string(&mut buf))?;
    if buf != "three" {
        return err("f3.txt has wrong data");
    }
    File::create("/tmp/merged/new.txt")
        .and_then(|mut f| f.write_all(b"Hello world!"))?;
    buf.clear();
    File::open("/tmp/merged/new.txt")
        .and_then(|mut f| f.read_to_string(&mut buf))?;
    if buf != "Hello world!" {
        return err("Can't read data just written in merge dir");
    }
    buf.clear();
    File::open("/tmp/dir3/new.txt")
        .and_then(|mut f| f.read_to_string(&mut buf))?;
    if buf != "Hello world!" {
        return err("Can't read data just written in upper dir");
    }
    Ok(())
}

pub fn check_overlayfs(wrapper: &Wrapper) -> i32 {
    if let Err(text) = setup_base_filesystem(
        wrapper.project_root, wrapper.ext_settings)
    {
        error!("Error setting base file system: {}", &text);
        return 122;
    }
    if let Err(err) = create_dirs() {
        error!("Couldn't create dirs: {}. It's probably not an issue \
            with overlayfs per se", err);
        return 2;
    }
    let mnt = Overlay::writable([
        "/tmp/dir1",
        "/tmp/dir2",
        ].iter().map(|x| x.as_ref()), "/tmp/dir3", "/tmp/wrk", "/tmp/merged");

    if let Err(err) = mnt.mount() {
        error!("{}", err);
        println!("unsupported");
        return 1;
    }
    if let Err(err) = check_read_write() {
        error!("overlay is mounted but got I/O error: {}", err);
        return 2;
    }
    println!("supported");
    return 0;
}
