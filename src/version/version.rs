use std::io::EndOfFile;
use std::io::fs::File;

use serialize::json;

use config::builders as B;
use container::sha256::Digest;


pub enum HashResult {
    Hashed,
    New,
    Error(String)
}


pub trait VersionHash {
    fn hash(&self, hash: &mut Digest) -> HashResult;
}


impl VersionHash for B::Builder {
    fn hash(&self, hash: &mut Digest) -> HashResult {
        match self {
            &B::Depends(ref filename) => {
                match
                    File::open(&Path::new("/work").join(filename))
                    .and_then(|mut f| {
                        loop {
                            let mut chunk = [0u8, .. 128*1024];
                            let bytes = match f.read(chunk) {
                                Ok(bytes) => bytes,
                                Err(ref e) if e.kind == EndOfFile => break,
                                Err(e) => return Err(e),
                            };
                            hash.input(chunk[..bytes]);
                        }
                        Ok(())
                    })
                {
                    Err(e) => return Error(format!("Can't read file: {}", e)),
                    Ok(()) => return Hashed,
                }
            }
            _ => {
                hash.input(json::encode(self).as_bytes());
                Hashed
            }
        }
    }
}
