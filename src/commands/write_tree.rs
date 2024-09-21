use std::{ffi::OsString, os::unix::fs::MetadataExt, path::Path};

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


pub(crate) fn recursive_tree_write(path: &Path) -> anyhow::Result<[u8;20]>{
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
    }
    // create tree object

    let tree_obj = Object::tree_obj_from_vec(vecc).context("couldnt create tree obj from entries vec")?;
    let hash = tree_obj.write_obj().context("couldnt write tree obj to .git/objects")?;

    Ok(hash)
}

// implementation of tree-write (excludes .git folder but nothing else)
pub(crate) fn invoke() ->anyhow::Result<()> {
    let hash = recursive_tree_write(std::path::Path::new("./"))?;
    println!("{}", hex::encode(hash));

    Ok(())
}
