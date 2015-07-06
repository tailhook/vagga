use std::fs::read_link;


pub fn container_ver(name: &String) -> Result<String, String> {
    let lnk = try!(read_link(&Path::new("/work/.vagga")
                             .join(name.as_slice()))
                   .map_err(|e| format!("Error reading link: {}", e)));
    let lnkcmp = lnk.str_components().collect::<Vec<Option<&str>>>();
    if lnkcmp.len() < 3 || lnkcmp[lnkcmp.len()-2].is_none() {
        return Err(format!("Broken container link"));
    }
    return Ok(lnkcmp[lnkcmp.len()-2].unwrap().to_string());
}
