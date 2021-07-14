use md5::{Digest, Md5};
use std::collections::HashMap;
use std::fs::{self, DirEntry, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

const BUFFER_SIZE: usize = 1024;

#[derive(Debug, StructOpt)]
#[structopt(name = "hashcmp", about = "find duplicate files")]
struct Opt {
    target_dir: String,
}

fn walk_dirs<F>(dir: &Path, cb: &mut F) -> io::Result<()>
where
    F: FnMut(&mut DirEntry) -> io::Result<()>,
{
    if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
            let mut entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk_dirs(&path, cb)?;
            } else {
                cb(&mut entry)?;
            }
        }
    }
    Ok(())
}

fn hash<D: Digest + Default, R: Read>(reader: &mut R) -> anyhow::Result<Vec<u8>> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = reader.read(&mut buffer)?;
        sh.update(&buffer[..n]);
        if n == 0 || n > BUFFER_SIZE {
            break;
        }
    }
    Ok(sh.finalize().to_ascii_uppercase())
}

fn build_table(target_dir: &Path, hash_db: &mut HashMap<Vec<u8>, Vec<PathBuf>>) -> io::Result<()> {
    walk_dirs(&target_dir, &mut |e: &mut DirEntry| {
        if let Ok(mut fh) = File::open(&e.path()) {
            let hashval = hash::<Md5, _>(&mut fh).unwrap();
            hash_db
                .entry(hashval)
                .or_insert(vec![PathBuf::from(e.path())]);
        }
        Ok(())
    })?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let mut hash_db: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();
    build_table(&Path::new(&opt.target_dir), &mut hash_db)?;
    for (key, val) in hash_db.iter() {
        println!("key: {:?} - val: {:?}", key, val);
    }
    Ok(())
}
