use std::collections::HashSet;
use std::fs::{File, Permissions, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[cfg(feature="containers")]
use zip::ZipArchive;
use quire::validate as V;

use crate::build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};
#[cfg(feature="containers")]
use crate::builder::context::Context;
#[cfg(feature="containers")]
use crate::capsule::download::maybe_download_and_check_hashsum;
#[cfg(feature="containers")]
use crate::file_util::{Dir, copy_stream};


#[derive(Serialize, Deserialize, Debug, Clone)]
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
        .member("path", V::Directory::new().absolute(true).default("/"))
        .member("subdir", V::Directory::new().default("").absolute(false))
    }
}


#[cfg(feature="containers")]
pub fn unzip_file(_ctx: &mut Context, src: &Path, dst: &Path,
    subdir: &Path)
    -> Result<(), String>
{
    let file = try_msg!(File::open(src),
        "Cannot open archive: {err}");

    let mut zip = try_msg!(ZipArchive::new(file),
        "Error when unpacking zip archive: {err}");

    let no_subdir = &subdir == &Path::new("") || &subdir == &Path::new(".");
    let mut dirs = HashSet::new();
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
            dirs.insert(fout_path.clone());
            try_msg!(Dir::new(&fout_path).recursive(true).create(),
                "Error creating dir: {err}");
        } else {
            let fout_base = fout_path.parent().unwrap();
            if !dirs.contains(fout_base) {
                try_msg!(Dir::new(&fout_base).recursive(true).create(),
                    "Error creating dir: {err}");
                dirs.insert(fout_base.to_path_buf());
            }
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
    fn name(&self) -> &'static str { "Unzip" }
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
            info!("Unzipping file: {}", self.url);

            let fpath = PathBuf::from("/vagga/root")
                .join(self.path.strip_prefix("/").unwrap());
            let (filename, _) = maybe_download_and_check_hashsum(
                &mut guard.ctx.capsule, &self.url, self.sha256.clone(),
                false)?;

            unzip_file(&mut guard.ctx, &filename, &fpath, &self.subdir)?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
