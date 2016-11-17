use std::path::{PathBuf};
use std::fs::{set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::ffi::OsStrExt;

use quire::validate as V;
use file_util::copy;
use builder::download::download_file;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(RustcDecodable, Debug)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub mode: u32,
}

impl Download {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("path", V::Directory::new().absolute(true))
        .member("mode", V::Numeric::new().default(0o644).min(0).max(0o1777))
    }
}


impl BuildStep for Download {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        hash.field("url", &self.url);
        hash.field("path", self.path.as_os_str().as_bytes());
        hash.text("mode", self.mode);
        Ok(())
    }
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            let fpath = PathBuf::from("/vagga/root")
                .join(self.path.strip_prefix("/").unwrap());
            let filename = if self.url.starts_with(".") {
                PathBuf::from("/work").join(&self.url)
            } else {
                download_file(&mut guard.ctx, &[&self.url], None)?
            };
            copy(&filename, &fpath)
                .map_err(|e| format!("Error copying {:?} to {:?}: {}",
                    &filename, self.path, e))?;
            set_permissions(&fpath, Permissions::from_mode(self.mode))
                .map_err(|e| format!("Error setting permissions for {:?}: {}",
                    self.path, e))?;
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
