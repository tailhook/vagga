use std::collections::treemap::TreeMap;
use super::Range;


#[deriving(Decodable)]
pub struct Container {

    pub builder: String,
    pub provision: Option<String>,
    pub parameters: TreeMap<String, String>,

    pub uids: Vec<Range>,
    pub gids: Vec<Range>,

    pub default_command: Option<Vec<String>>,
    pub command_wrapper: Option<Vec<String>>,
    pub shell: Vec<String>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
    pub tmpfs_volumes: TreeMap<String, String>,  // volumes
}

impl PartialEq for Container {
    fn eq(&self, _other: &Container) -> bool { false }
}

