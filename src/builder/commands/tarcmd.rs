use std::cmp::min;
use std::fs::{File, Permissions};
use std::fs::{create_dir_all, set_permissions, hard_link};
use std::io::{self, Read, Seek, SeekFrom, BufReader};
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::u32;

#[cfg(feature="containers")] use bzip2::read::BzDecoder;
#[cfg(feature="containers")] use flate2::read::GzDecoder;
#[cfg(feature="containers")] use libmount::BindMount;
#[cfg(feature="containers")] use tar::Archive;
#[cfg(feature="containers")] use xz2::read::XzDecoder;
use quire::validate as V;

#[cfg(feature="containers")]
use crate::{
    build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard},
    builder::commands::generic::run_command_at,
    builder::context::Context,
    builder::dns::revert_name_files,
    capsule::download::{maybe_download_and_check_hashsum},
    container::mount::{unmount},
    file_util::{Dir, read_visible_entries, copy_stream, set_owner_group},
};


#[derive(Serialize, Deserialize, Debug)]
pub struct Tar {
    pub url: String,
    pub sha256: Option<String>,
    pub path: PathBuf,
    pub subdir: PathBuf,
}

impl Tar {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("sha256", V::Scalar::new().optional())
        .member("path", V::Directory::new().absolute(true).default("/"))
        .member("subdir", V::Directory::new().default("").absolute(false))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TarInstall {
    pub url: String,
    pub sha256: Option<String>,
    pub subdir: Option<PathBuf>,
    pub script: String,
}

impl TarInstall {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("sha256", V::Scalar::new().optional())
        .member("subdir", V::Directory::new().optional().absolute(false))
        .member("script", V::Scalar::new()
                .default("./configure --prefix=/usr\n\
                          make\n\
                          make install\n"))
    }
}


#[cfg(feature="containers")]
pub fn unpack_file(_ctx: &mut Context, src: &Path, tgt: &Path,
    includes: &[&Path], excludes: &[&Path], preserve_owner: bool)
    -> Result<(), String>
{

    info!("Unpacking {:?} -> {:?}", src, tgt);
    let read_err = |e| format!("Error reading {:?}: {}", src, e);

    let mut file = BufReader::new(File::open(&src).map_err(&read_err)?);

    let mut buf = [0u8; 8];
    let nbytes = file.read(&mut buf).map_err(&read_err)?;
    file.seek(SeekFrom::Start(0)).map_err(&read_err)?;
    let magic = &buf[..nbytes];
    if magic.len() >= 2 && magic[..2] == [0x1f, 0x8b] {
        return unpack_stream(GzDecoder::new(file),
            src, tgt, includes, excludes, preserve_owner);
    } else if magic.len() >= 6 && magic[..6] ==
        [ 0xFD, b'7', b'z', b'X', b'Z', 0x00]
    {
        return unpack_stream(XzDecoder::new(file),
            src, tgt, includes, excludes, preserve_owner);
    } else if magic.len() >= 3 && magic[..3] == [ b'B', b'Z', b'h'] {
        return unpack_stream(BzDecoder::new(file),
            src, tgt, includes, excludes, preserve_owner);
    } else {
        return Err(format!("unpacking {:?}: unexpected compression", src));
    }

}

#[cfg(feature="containers")]
fn unpack_stream<F: Read>(file: F, srcpath: &Path, tgt: &Path,
    includes: &[&Path], excludes: &[&Path], preserve_owner: bool)
    -> Result<(), String>
{
    let read_err = |e| format!("Error reading {:?}: {}", srcpath, e);
    let mut arc = Archive::new(file);
    let mut hardlinks = Vec::new();

    for item in arc.entries().map_err(&read_err)? {
        let mut src = item.map_err(&read_err)?;
        let path_ref = src.path().map_err(&read_err)?
            .to_path_buf();
        let mut orig_path: &Path = &path_ref;
        if orig_path.is_absolute() {
            orig_path = orig_path.strip_prefix("/").unwrap();
        }
        if includes.len() > 0 {
            if !includes.iter().any(|x| orig_path.starts_with(x)) {
                continue;
            }
        }
        if excludes.iter().any(|x| orig_path.starts_with(x)) {
            continue;
        }
        let path = tgt.join(orig_path);
        let write_err = |e| format!("Error writing {:?}: {}", path, e);
        let entry = src.header().entry_type();

        // Some archives don't have uids
        // TODO(tailhook) should this be handled in tar-rs?
        let uid = min(src.header().uid().unwrap_or(0), u32::MAX as u64) as u32;
        let gid = min(src.header().gid().unwrap_or(0), u32::MAX as u64) as u32;

        if entry.is_dir() {
            let mode = src.header().mode().map_err(&read_err)?;
            let mut dir_builder = Dir::new(&path);
            dir_builder.recursive(true).mode(mode);
            if preserve_owner {
                dir_builder.uid(uid).gid(gid);
            }
            dir_builder.create().map_err(&write_err)?;
        } else if entry.is_symlink() {
            let src = src.link_name().map_err(&read_err)?
                .ok_or(format!("Error unpacking {:?}, broken symlink", path))?;
            match symlink(&src, &path) {
                Ok(_) => {},
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        if let Some(parent) = path.parent() {
                            Dir::new(parent).recursive(true).create()
                                .map_err(&write_err)?;
                            symlink(&src, &path).map_err(&write_err)?
                        } else {
                            return Err(write_err(e));
                        }
                    } else {
                        return Err(write_err(e));
                    }
                }
            };
        } else if entry.is_hard_link() {
            let link = src.link_name().map_err(&read_err)?
                .ok_or(format!("Error unpacking {:?}, broken symlink", path))?;
            let link = if link.is_absolute() {
                link.strip_prefix("/").unwrap()
            } else {
                &*link
            };
            hardlinks.push((tgt.join(link).to_path_buf(), path.to_path_buf()));
        } else if entry.is_pax_global_extensions() ||
                  entry.is_pax_local_extensions() ||
                  entry.is_gnu_longname() ||
                  entry.is_gnu_longlink()
        {
            // nothing to do
        } else {
            let mut dest = match File::create(&path) {
                Ok(x) => x,
                Err(e) => {
                    if e.kind() == io::ErrorKind::NotFound {
                        if let Some(parent) = path.parent() {
                            Dir::new(parent).recursive(true).create()
                                .map_err(&write_err)?;
                            File::create(&path).map_err(&write_err)?
                        } else {
                            return Err(write_err(e));
                        }
                    } else {
                        return Err(write_err(e));
                    }
                }
            };
            copy_stream(&mut src, &mut dest).map_err(|e|
                format!("Error unpacking {:?} -> {:?}: {}",
                        srcpath, path, e))?;
            let mode = src.header().mode().map_err(&read_err)?;
            set_permissions(&path, Permissions::from_mode(mode))
                .map_err(&write_err)?;
            if preserve_owner {
                set_owner_group(&path, uid, gid).map_err(&write_err)?;
            }
        }
    }
    for (src, dst) in hardlinks.into_iter() {
        let write_err = |e| {
            format!("Error hardlinking {:?} - {:?}: {}", &src, &dst, e)
        };
        match hard_link(&src, &dst) {
            Ok(_) => {},
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    if let Some(parent) = dst.parent() {
                        Dir::new(parent).recursive(true).create()
                            .map_err(&write_err)?;
                        hard_link(&src, &dst).map_err(&write_err)?
                    } else {
                        return Err(write_err(e));
                    }
                } else {
                    return Err(write_err(e));
                }
            }
        };
    }
    Ok(())
}

#[cfg(feature="containers")]
pub fn unpack_subdir(ctx: &mut Context, filename: &Path, dest: &Path,
    subdir: &Path)
    -> Result<(), String>
{
    let tmppath = PathBuf::from("/vagga/root/tmp")
        .join(filename.file_name().unwrap());
    let tmpsub = tmppath.join(subdir);
    try_msg!(Dir::new(&tmpsub).recursive(true).create(),
        "Error making dir: {err}");
    if !dest.exists() {
        try_msg!(Dir::new(&dest).recursive(true).create(),
            "Error making dir: {err}");
    }
    try_msg!(BindMount::new(&dest, &tmpsub).mount(),
        "temporary tar mount: {err}");
    let res = if subdir == Path::new("") {
        unpack_file(ctx, &filename, &tmppath, &[], &[], false)
    } else {
        unpack_file(ctx, &filename, &tmppath, &[subdir], &[], false)
    };
    unmount(&tmpsub)?;
    res
}

#[cfg(feature="containers")]
pub fn tar_command(ctx: &mut Context, tar: &Tar) -> Result<(), String>
{
    let fpath = PathBuf::from("/vagga/root")
        .join(tar.path.strip_prefix("/").unwrap());
    let (filename, _) = maybe_download_and_check_hashsum(
        &mut ctx.capsule, &tar.url, tar.sha256.clone(), false)?;

    if &Path::new(&tar.subdir) == &Path::new(".") {
        unpack_file(ctx, &filename, &fpath, &[], &[], false)?;
    } else {
        unpack_subdir(ctx, &filename, &fpath, &tar.subdir)?;
    }
    if tar.path == Path::new("/") {
        revert_name_files()?;
    }
    Ok(())
}

#[cfg(feature="containers")]
pub fn tar_install(ctx: &mut Context, tar: &TarInstall)
    -> Result<(), String>
{
    let (filename, _) = maybe_download_and_check_hashsum(
        &mut ctx.capsule, &tar.url, tar.sha256.clone(), false)?;

    let tmppath = PathBuf::from("/vagga/root/tmp")
        .join(filename.file_name().unwrap());
    create_dir_all(&tmppath)
         .map_err(|e| format!("Error making dir: {}", e))?;
    set_permissions(&tmppath, Permissions::from_mode(0o755))
         .map_err(|e| format!("Error setting permissions: {}", e))?;
    unpack_file(ctx, &filename, &tmppath, &[], &[], false)?;
    let workdir = if let Some(ref subpath) = tar.subdir {
        tmppath.join(subpath)
    } else {
        let items = read_visible_entries(&tmppath)
            .map_err(|e| format!("Error reading dir: {}", e))?;
        if items.len() != 1 {
            if items.len() == 0 {
                return Err("Tar archive was empty".to_string());
            } else {
                return Err("Multiple directories was unpacked. \
                    If thats expected use `subdir: \".\"` or any \
                    other directory".to_string());
            }
        }
        items.into_iter().next().unwrap()
    };
    let workdir = PathBuf::from("/").join(
        workdir.strip_prefix("/vagga/root").unwrap());
    return run_command_at(ctx, &[
        "/bin/sh".to_string(),
        "-exc".to_string(),
        tar.script.to_string()],
        &workdir);
}

impl BuildStep for Tar {
    fn name(&self) -> &'static str { "Tar" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        if let Some(ref sha) = self.sha256 {
            hash.field("hash", sha);
        } else {
            hash.field("url", &self.url);
        }
        hash.field("path", &self.path);
        hash.field("subdir", &self.subdir);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            tar_command(&mut guard.ctx, self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}

impl BuildStep for TarInstall {
    fn name(&self) -> &'static str { "TarInstall" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        if let Some(ref sha) = self.sha256 {
            hash.field("hash", sha);
        } else {
            hash.field("url", &self.url);
        }
        hash.opt_field("subdir", &self.subdir);
        hash.field("script", &self.script);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            tar_install(&mut guard.ctx, self)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
