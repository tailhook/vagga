use std::io::stdio::{stdout, stderr};
use std::io::fs::File;

use serialize::json::from_reader;
use J = serialize::json;
use argparse::{ArgumentParser, Store, List};

use super::super::env::Environ;


pub fn extract_json(_env: &mut Environ, args: Vec<String>)
    -> Result<int, String>
{
    let mut filename = Path::new(".");
    let mut columns = Vec::new();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut filename)
            .add_argument("filename", box Store::<Path>,
                "A JSON file to parse")
            .required();
        ap.refer(&mut columns)
            .add_argument("column", box List::<String>,
                "A columns to extract");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Ok(0),
            Err(_) => return Ok(122),
        }
    }
    let mut file = try!(File::open(&filename)
        .map_err(|e| format!("Can't open file {}: {}",
                             filename.display(), e)));
    let json = try!(from_reader(&mut file)
        .map_err(|e| format!("Can't parse {}: {}", filename.display(), e)));
    let lst = match json {
        J::List(lst) => lst,
        _ => return Err(format!("Root entitity is not a list")),
    };
    let mut out = stdout();
    for obj in lst.iter() {
        let tree = match obj {
            &J::Object(ref tmap) => tmap,
            _ => return Err(format!("Not an object in a list")),
        };
        for col in columns.iter() {
            match tree.find(col) {
                Some(&J::String(ref x)) => try!(out.write_str(x.as_slice())
                    .map_err(|e| format!("Error writing to stdout: {}", e))),
                _ => {},
            };
            try!(out.write_char('\t')
                .map_err(|e| format!("Error writing to stdout: {}", e)));
        }
        try!(out.write_char('\n')
            .map_err(|e| format!("Error writing to stdout: {}", e)));
    }
    return Ok(0);
}
