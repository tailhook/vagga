use std::io::Write;
use std::fs::{File, set_permissions, Permissions};
use std::path::{PathBuf, Path};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;

use quire::validate as V;
use build_step::{BuildStep, VersionError, StepError, Digest, Config, Guard};


#[derive(Debug, Serialize, Deserialize)]
pub struct Text(BTreeMap<PathBuf, String>);

impl Text {
    pub fn config() -> V::Mapping<'static> {
        V::Mapping::new(
            V::Directory::new().absolute(true),
            V::Scalar::new())
    }
}

impl BuildStep for Text {
    fn name(&self) -> &'static str { "Text" }
    #[cfg(feature="containers")]
    fn hash(&self, _cfg: &Config, hash: &mut Digest)
        -> Result<(), VersionError>
    {
        for (k, v) in &self.0 {
            hash.field("path", k);
            hash.field("data", v);
        }
        Ok(())
    }
    #[cfg(feature="containers")]
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
