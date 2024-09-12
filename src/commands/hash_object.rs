use crate::objects::Object;
use anyhow::{Context, Ok};
use std::path::Path;

pub(crate) fn invoke(save: bool, file: &Path) -> anyhow::Result<()> {

    let blob = Object::blob_from_file(file).context("opening blob from file")?;

    let hash = if save {
        blob
            .write_obj()
            .context("write file into blob object file")?
    } else {
        blob
            .write(std::io::sink())
            .context("write file into blob object")?
    };

    println!("{}", hex::encode(hash));
    Ok(())
}
