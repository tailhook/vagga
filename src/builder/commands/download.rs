use std::path::{PathBuf};
use std::fs::{set_permissions, Permissions};
use std::os::unix::fs::PermissionsExt;

use quire::validate as V;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(Deserialize, Debug)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub mode: u32,
    pub sha256: Option<String>,
}

impl Download {
    pub fn config() -> V::Structure<'static> {
        V::Structure::new()
        .member("url", V::Scalar::new())
        .member("path", V::Directory::new().absolute(true))
        .member("mode", V::Numeric::new().default(0o644).min(0).max(0o1777))
        .member("sha256", V::Scalar::new().optional())
    }
}


impl BuildStep for Download {
    fn name(&self) -> &'static str { "Download" }
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
        hash.field("mode", self.mode);
        Ok(())
    }
    #[cfg(feature="containers")]
    fn build(&self, guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        use capsule::download::maybe_download_and_check_hashsum;
        use file_util::copy;

        if build {
            let fpath = PathBuf::from("/vagga/root")
                .join(self.path.strip_prefix("/").unwrap());
            let (filename, _) = maybe_download_and_check_hashsum(
                &mut guard.ctx.capsule, &self.url,
                self.sha256.clone(), false)?;
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
