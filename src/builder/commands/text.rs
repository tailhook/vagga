use std::io::Write;
use std::fs::{File, set_permissions, Permissions};
use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::ffi::OsStrExt;

use quire::validate as V;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(Debug)]
pub struct Text(BTreeMap<PathBuf, String>);
tuple_struct_decode!(Text);

impl Text {
    pub fn config() -> V::Mapping<'static> {
        V::Mapping::new(
            V::Directory::new().is_absolute(true),
            V::Scalar::new())
    }
}

impl BuildStep for Text {
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        for (k, v) in &self.0 {
            hash.field(k.as_os_str().as_bytes(), v);
        }
        Ok(())
    }
    fn build(&self, _guard: &mut Guard, build: bool)
        -> Result<(), StepError>
    {
        if build {
            for (path, text) in self.0.iter() {
                let realpath = Path::new("/vagga/root")
                    .join(path.strip_prefix("/").unwrap());
                File::create(&realpath)
                    .and_then(|mut f| f.write_all(text.as_bytes()))
                    .map_err(|e| format!("Can't create file: {}", e))?;
                set_permissions(&realpath, Permissions::from_mode(0o644))
                    .map_err(|e| format!("Can't chmod file: {}", e))?;
            }
        }
        Ok(())
    }
    fn is_dependent_on(&self) -> Option<&str> {
        None
    }
}
