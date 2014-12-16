use libc::{uid_t, gid_t};


pub struct Settings {
    pub version_check: bool,
    pub uid_map: Option<(Vec<(uid_t, uid_t, uid_t)>,
                         Vec<(gid_t, gid_t, gid_t)>)>,
}
