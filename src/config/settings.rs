use std::str::FromStr;
use serialize::json;
use libc::{uid_t, gid_t};


#[derive(Decodable, Encodable, Default, Clone)]
pub struct Settings {
    pub version_check: bool,
    pub ubuntu_mirror: String,
    pub alpine_mirror: Option<String>,
    pub uid_map: Option<(Vec<(uid_t, uid_t, uid_t)>,
                         Vec<(gid_t, gid_t, gid_t)>)>,
}

impl FromStr for Settings {
    type Err = ();
    fn from_str(val: &str) -> Result<Settings, ()> {
        json::decode(val).map_err(|_| ())
    }
}

