use std::env;

use anyhow::Context;

use crate::objects::Object;

pub(crate) fn write_commit(message: &str, tree_hash: &str, parent_hash: Option<&str>) -> anyhow::Result<[u8;20]> {
    let (name, email) =
        if let (Some(name), Some(email)) = (env::var_os("NAME"), env::var_os("EMAIL")) {
            let name = name
                .into_string()
                .map_err(|_| anyhow::anyhow!("$NAME is invalid utf-8"))?;
            let email = email
                .into_string()
                .map_err(|_| anyhow::anyhow!("$EMAIL is invalid utf-8"))?;
            (name, email)
        } else {
            (
                String::from("cerdelen"),
                String::from("cerdelen@cerdelen.com"),
            )
        };

    let obj = Object::commit_obj(tree_hash, &name, &email, message, parent_hash).context("making commit object")?;
    let hash = obj.write_obj().context("couldnt write tree obj to .git/objects")?;
    Ok(hash)
}

pub(crate) fn invoke(parent_hash: Option<String>, commit_message: String, tree_hash: String) ->anyhow::Result<()> {
    let hash = write_commit(&commit_message, &tree_hash, parent_hash.as_deref())?;

    println!("{}", hex::encode(hash));

    Ok(())
}
