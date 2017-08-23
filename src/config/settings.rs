use std::str::FromStr;
use std::collections::BTreeMap;

use libc::{uid_t, gid_t};
use serde_json;


const DEFAULT_UBUNTU_MIRROR: &str = "mirror://mirrors.ubuntu.com/mirrors.txt";

const DEFAULT_ALPINE_MIRROR: &str = "http://dl-cdn.alpinelinux.org/alpine/";

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
pub struct Settings {
    pub version_check: bool,
    pub proxy_env_vars: bool,
    pub ubuntu_mirror: Option<String>,
    pub alpine_mirror: Option<String>,
    pub uid_map: Option<(Vec<(uid_t, uid_t, uid_t)>,
                         Vec<(gid_t, gid_t, gid_t)>)>,
    pub push_image_script: Option<String>,
    pub build_lock_wait: bool,
    pub auto_apply_sysctl: bool,
    pub environ: BTreeMap<String, String>,
    pub index_all_images: bool,
    pub hard_link_identical_files: bool,
    pub run_symlinks_as_commands: bool,
    pub disable_auto_clean: bool,
    pub storage_subdir_from_env_var: Option<String>,
}

impl Settings {
    pub fn ubuntu_mirror(&self) -> &str {
        self.ubuntu_mirror.as_ref().map(|m| m.as_str())
            .unwrap_or(DEFAULT_UBUNTU_MIRROR)
    }

    pub fn alpine_mirror(&self) -> &str {
        self.alpine_mirror.as_ref().map(|m| m.as_str())
            .unwrap_or(DEFAULT_ALPINE_MIRROR)
    }
}

impl FromStr for Settings {
    type Err = ();
    fn from_str(val: &str) -> Result<Settings, ()> {
        serde_json::from_str(val).map_err(|_| ())
    }
}
