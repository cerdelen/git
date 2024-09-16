use core::panic;
use std::{ffi::{OsStr, OsString}, io::BufRead, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};

use anyhow::Context;

use crate::objects::Object;

struct TreeEntry<'a, R> {
    // this cannot be not a reference because of not knowing size beforehand
    name: &'a OsStr,
    obj: Object<R>,
    // rights:
}


fn recursive_tree_write(path: &Path) -> anyhow::Result<[u8;20]>{
    // can put name of file into error message
    for entry in std::fs::read_dir(path).context("reading curr dir")? {
        let mut vecc: Vec<&OsStr>= Vec::new();
        let entry = entry.context("entry is error")?;
        let path = entry.path();
        let temp_name = path.file_name().unwrap();
        vecc.push(temp_name);
        if let Some(dir_name) = path.file_name() {
            if dir_name== ".git" || dir_name == "target" {
                continue;
            }
        }
        let mode: u32;
        let kind: &str;
        let mut hash: [u8;20] = [0;20];
        if path.is_file() {
            let blob = Object::blob_from_file(entry.path()).context("making blob from dir entry")?;
            hash = blob.write_obj().context("write blob into .git/objects")?;
            mode = entry.metadata().unwrap().mode();
            kind = "blob";
        } else if path.is_dir() {
            recursive_tree_write(&path)?;
            mode = 0o040000;
            kind = "tree";
            // println!("directory: {}", entry.path().display());
        } else {
            anyhow::bail!("what even is this entry type? {:?}", entry);
        }
        println!("{:06o} {} {} {:?}", mode, kind, hex::encode(hash), entry.file_name());
    }
    // create tree object
    println!("end of a dir {:?}", path.file_name());
    Ok([1;20])
}

// implementation of tree-write (excludes .git folder but nothing else)
pub(crate) fn invoke() ->anyhow::Result<()> {

    println!("invoked tree write");
    // i have to recursivley create hashes for all directories i find in my dir


    // read curr dir
    // iter over all entries
    //      if file -> blob and store it in temp vec
    //      if dir recursively do all of this
    //      from temp vec with recursively read dirs and all blobs create tree obj
    //      (find out how to sort this temp vec so the tree obj are right order and hashes are same
    //      as in git)
    //      apparently alphabetically by the name of file/dir
    //       vec.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let hash = recursive_tree_write(std::path::Path::new("./"))?;
    println!("{}", hex::encode(hash));

    Ok(())
}
