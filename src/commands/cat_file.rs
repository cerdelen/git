use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(pretty: bool, obj_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty,
        "-p flag must be given with the cat-file command, non pretty mode not supported"
    );

    let mut obj = Object::read(obj_hash).context("parse blob from file")?;
    match obj.kind {
        Kind::Blob => {
            let mut stdout = std::io::stdout().lock();
            let n = std::io::copy(&mut obj.reader, &mut stdout)
                .context("write .git/objects cat-file to stdout")?;

            anyhow::ensure!(
                n == obj.expected_size,
                ".git/objects files expected size != actual size: expected {}, actual {n}",
                obj.expected_size,
            );
        },
        _ => anyhow::bail!("not a blob which was tried to be cat-filed '{}'", obj.kind),
    }

    Ok(())
}
