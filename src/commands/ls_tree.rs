use anyhow::Context;
use crate::objects::{Kind, Object};

pub(crate) fn invoke(_nane_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut obj = Object::read(tree_hash).context("parse blob from file")?;

    match obj.kind {
        Kind::Tree => {
            println!("found tree");
            // entries are stores as follows
            // <mode> <name>\0<sha>
            // so i gotta loop until im done reading and always split until 0
            // and read into a buffer of length [u8,20] which i can somehow decode then
            // let hash = hex::encode(&hashbuf); like this??
            let mut stdout = std::io::stdout().lock();
            let _n = std::io::copy(&mut obj.reader, &mut stdout)
                .context("write .git/objects cat-file to stdout")?;
        },
        _ => println!("Not a Tree Object!"),
    };

    Ok(())
}
