use std::str::FromStr;

use libc::{uid_t, gid_t};
use rustc_serialize::json;


#[derive(RustcDecodable, RustcEncodable, Default, Clone, Debug)]
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
}

impl FromStr for Settings {
    type Err = ();
    fn from_str(val: &str) -> Result<Settings, ()> {
        json::decode(val).map_err(|_| ())
    }
}

