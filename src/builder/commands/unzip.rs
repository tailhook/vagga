use std::fs::{File, Permissions, set_permissions};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

use quire::validate as V;
use builder::context::Context;
use builder::download::maybe_download_and_check_hashsum;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
use file_util::{copy_stream, create_dir};
use path_util::ToRelative;


#[derive(RustcEncodable, RustcDecodable, Debug, Clone)]
pub struct Unzip {
    pub url: String,
    pub sha256: Option<String>,
    pub path: PathBuf,
    pub subdir: PathBuf,
}

impl Unzip {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("sha256", V::Scalar::new().optional())
        .member("path", V::Directory::new().is_absolute(true).default("/"))
        .member("subdir", V::Directory::new().default("").is_absolute(false))
    }
}


pub fn unzip_file(_ctx: &mut Context, src: &Path, dst: &Path,
    subdir: &Path)
    -> Result<(), String>
{
    let file = try_msg!(File::open(src),
        "Cannot open archive: {err}");

    let mut zip = try_msg!(ZipArchive::new(file),
        "Error when unpacking zip archive: {err}");

    let no_subdir = &subdir == &Path::new("") || &subdir == &Path::new(".");
    let mut found_subdir = no_subdir;
    for i in 0..zip.len() {
        let mut fin = zip.by_index(i).unwrap();
        let fin_name = String::from(fin.name());
        let fin_path = PathBuf::from(&fin_name);
        let fout_path = if no_subdir {
            dst.join(&fin_path)
        } else if fin_path.starts_with(subdir) {
            found_subdir = true;
            dst.join(try_msg!(fin_path.strip_prefix(subdir), "{err}"))
        } else {
            continue;
        };
        if &fout_path == &Path::new("") {
            continue;
        }
        if fin_name.ends_with("/") {
            try_msg!(create_dir(&fout_path, true),
                "Error creating dir: {err}");
        } else {
            let mut fout = try_msg!(File::create(&fout_path),
                "Error creating file: {err}");
            try_msg!(copy_stream(&mut fin, &mut fout),
                "Error unpacking file: {err}");
        }
        if let Some(mode) = fin.unix_mode() {
            let perms = Permissions::from_mode(mode);
            try_msg!(set_permissions(&fout_path, perms),
                 "Error setting permissions: {err}");
        }
    }

    if found_subdir {
        Ok(())
    } else {
        Err(format!("{:?} is not found in archive", subdir))
    }
}

impl BuildStep for Unzip {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        if let Some(ref sha) = self.sha256 {
            hash.field("hash", sha);
        } else {
            hash.field("url", &self.url);
        }
        hash.field("path", self.path.as_os_str().as_bytes());
        hash.field("subdir", self.subdir.as_os_str().as_bytes());
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            info!("Unzipping file: {}", self.url);

            let fpath = PathBuf::from("/vagga/root").join(self.path.rel());
            let filename = try!(maybe_download_and_check_hashsum(
                &mut guard.ctx, &self.url, self.sha256.clone()));

            try!(unzip_file(&mut guard.ctx, &filename, &fpath, &self.subdir));
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
