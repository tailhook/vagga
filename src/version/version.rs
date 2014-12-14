use std::io::EndOfFile;
use std::path::BytesContainer;
use std::io::fs::File;

use config::builders as B;
use container::sha256::Digest;
use container::root::temporary_change_root;


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
            &B::UbuntuCore(ref name) => {
                // TODO(tailhook) get hash of the downloaded image
                debug!("Add to hash `UbuntuCore:{}`", name);
                hash.input("UbuntuCore:".as_bytes());
                hash.input(name.as_bytes());
                hash.input(&[0]);
                Hashed
            }
            &B::Cmd(ref vec) => {
                vec.iter().all(|cmd| {
                    hash.input(cmd.as_bytes());
                    hash.input(&[0]);
                    true
                });
                Hashed
            }
            &B::Sh(ref cmd) => {
                hash.input(cmd.as_bytes());
                hash.input(&[0]);
                Hashed
            }
            &B::Env(ref pairs) => {
                for (k, v) in pairs.iter() {
                    hash.input(k.as_bytes());
                    hash.input(&[0]);
                    hash.input(v.as_bytes());
                    hash.input(&[0]);
                }
                Hashed
            }
            &B::Remove(ref path) | &B::EnsureDir(ref path) |
            &B::EmptyDir(ref path) => {
                hash.input(path.container_as_bytes());
                hash.input(&[0]);
                Hashed
            }
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
            &B::Tar(ref tar) => {
                hash.input(tar.url.as_bytes());
                hash.input(&[0]);
                tar.sha256.as_ref().map(|x| hash.input(x.as_bytes()));
                hash.input(&[0]);
                hash.input(tar.path.container_as_bytes());
                hash.input(&[0]);
                Hashed
            }
        }
    }
}
