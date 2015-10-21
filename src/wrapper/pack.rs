use argparse::{ArgumentParser, Store, List, StoreTrue};


enum Compression {
    Detect,
    Gzip,
    Xz,
}


pub fn pack_image_cmd(wrapper: &Wrapper, cmdline: Vec<String>)
    -> Result<i32, String>
{
    let mut container: String = "".to_string();
    let mut target: String = "".to_string();
    let mut compression: Compression = Detect;
    {

        ap.refer(&mut container)
            .add_argument("container_name", Store,
                "Container name to pack");
        ap.refer(&mut compression)
            .add_option(&["-z", "--gzip"], StoreConst(Gzip),
                "Compress with gzip compression")
            .add_option(&["-J", "--xz"], StoreConst(Xz),
                "Compress with xz compression");
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => {
                return Ok(122);
            }
        }
    }
}
