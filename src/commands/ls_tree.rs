use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

use crate::objects::{Kind, Object};
use anyhow::Context;

pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut obj = Object::read(tree_hash).context("parse blob from file")?;

    match obj.kind {
        Kind::Tree => {
            let mut buf = Vec::new();
            let mut hash_buf = [0; 20];
            let mut stdout = std::io::stdout().lock();
            loop {
                if obj .reader .read_until(0, &mut buf) .context("reading object")? == 0 { break; }
                obj.reader
                    .read_exact(&mut hash_buf[..])
                    .context("reading 20 bytes for hash")?;

                let mode_name = CStr::from_bytes_with_nul(&buf).unwrap();
                let mut splits = mode_name.to_bytes().splitn(2, |&b| b == b' ');
                let mode = splits.next().expect("slpit has always at least 1 output");
                let name = splits
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("tree entry has no file name"))?;

                match name_only {
                    true => stdout
                        .write_all(name)
                        .context("write file name in name only mode")?,
                    false => {
                        let mode = std::str::from_utf8(mode).context("mode wasnt valid utf8")?;
                        let ptd_obj_hash = hex::encode(&hash_buf);
                        let pointed_obj = Object::read(&ptd_obj_hash)
                            .with_context(|| format!("read obj for entry {ptd_obj_hash} failed"))?;
                        write!(stdout, "{mode:0>6} {} {ptd_obj_hash} ", pointed_obj.kind)
                            .context("write tree entry meta data to stdout")?;
                        stdout
                            .write_all(name)
                            .context("write tree entry name to stdout")?;
                    }
                };
                buf.clear();
                writeln!(stdout, "").context("new line to stdout")?;
            } // loop
        } // match
        _ => anyhow::bail!("Not a Tree obj but: {}", obj.kind),
    };
    Ok(())
}
