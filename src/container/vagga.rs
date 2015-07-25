use std::fs::read_link;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;


pub fn container_ver(name: &String) -> Result<String, String> {
    let lnk = try!(read_link(&Path::new("/work/.vagga").join(&name))
                   .map_err(|e| format!("Error reading link: {}", e)));
    let lnkcmp = lnk.iter().collect::<Vec<_>>();
    if lnkcmp.len() < 3 || lnkcmp[lnkcmp.len()-2].to_str().is_none() {
        return Err(format!("Broken container link"));
    }
    return Ok(lnkcmp[lnkcmp.len()-2].to_str().unwrap().to_string());
}
