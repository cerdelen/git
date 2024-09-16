use core::panic;
use std::{ffi::{OsStr, OsString}, io::{BufRead, Write}, os::unix::fs::{MetadataExt, PermissionsExt}, path::{Path, PathBuf}};

use anyhow::Context;

use crate::objects::{Kind, Object};

#[derive(Debug)]
pub(crate) struct TreeEntry {
    // this cannot be not a reference because of not knowing size beforehand
    pub(crate) name: OsString,
    pub(crate) mode: u32,
    pub(crate) hash: [u8;20],
    pub(crate) kind: Kind,
    // rights:
}


fn recursive_tree_write(path: &Path) -> anyhow::Result<[u8;20]>{
    let dir = std::fs::read_dir(path).context("reading curr dir")? ;
    let mut vecc: Vec<TreeEntry>= Vec::new();
    for entry in dir {
        let entry = entry.context("entry is error")?;
        let path = entry.path();
        if let Some(dir_name) = path.file_name() {
            if dir_name== ".git" || dir_name == "target" {
                continue;
            }
        }
        let tree_entry: TreeEntry = if path.is_file() {
            let blob = Object::blob_from_file(entry.path()).context("making blob from dir entry")?;
            TreeEntry {
                name: entry.file_name(),
                mode: entry.metadata().unwrap().mode(),
                hash: blob.write_obj().context("write blob into .git/objects")?,
                kind: Kind::Blob,
            }
        } else if path.is_dir() {
            let hash = recursive_tree_write(&path)?;
            TreeEntry {
                name: entry.file_name(),
                mode: 0o040000,
                hash,
                kind: Kind::Tree,
            }
        } else {
            anyhow::bail!("what even is this entry type? {:?}", entry);
        };
        vecc.push(tree_entry);
        // println!("{:06o} {} {} {:?}", tree_entry.mode, tree_entry.kind, hex::encode(tree_entry.hash), entry.file_name());
    }
    // create tree object

    let tree_obj = Object::tree_obj_from_vec(&vecc).context("couldnt create tree obj from entries vec")?;
    let hash = tree_obj.write_obj().context("couldnt write tree obj to .git/objects")?;

    Ok(hash)
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
