use std::io::{BufRead, Read};

use anyhow::Context;

use crate::objects::Object;

struct Commit {
    author: String,
    date: String,
    message: String,
    parent: Option<[u8;20]>,
}

impl Commit {
    fn from_obj(mut obj: Object<impl BufRead>) -> anyhow::Result<Commit>{
        let mut content = String::new();

        let mut buf = Vec::new();
        let mut parent_or_author_buf = Vec::new();
        let mut author_buf = Vec::new();
        let mut message = Vec::new();
        // obj.reader.read_to_string(&mut content);
        let mut commit  = Commit {
            author: String::new(),
            date: String::new(),
            message: String::new(),
            parent: None,
        };

        // tree
        obj.reader.read_until(b'\n', &mut buf).context("parsing tree from commit")?;
        // let mut parent_line = String::new();
        obj.reader.read_until(b'\n', &mut parent_or_author_buf).context("parsing tree from commit")?;
        let mut splits = parent_or_author_buf.splitn(2, |&b| b == b' ');
        let line_type = splits.next().expect("split has always at least 1 output");
        println!("praent or author buf == {:?}", parent_or_author_buf);
        println!("line_type == {:?}", line_type);
        if line_type == "parent".as_bytes() {
            let parent_slice = splits.next().context("parent line has no hash")?;
            commit.parent = Some(parent_slice[0..20].try_into().context("parent_hash not big enough")?);
            obj.reader.read_until(b'\n', &mut author_buf).context("parsing tree from commit")?;
            println!("found parent hash: {}", &hex::encode(commit.parent.unwrap()));
        } else {
            commit.parent = None;
        }
        // obj.reader.read_line(&mut parent_line).context("parsing parent_line from commit")?;
        // obj.reader.read_line(&mut commit.author).context("parsing author from commit")?;
        // commiter
        obj.reader.read_until(b'\n', &mut buf).context("parsing commiter from commit")?;
        obj.reader.read_until(b'\n', &mut buf).context("parsing empty line from commit")?;
        obj.reader.read_until(b'\n', &mut message).context("parsing message from commit")?;
        println!("content:\n{}", content);
        println!("{:?} {:?} {:?} {:?}", hex::encode(commit.parent.unwrap()), buf, author_buf, message);

        // Commit {
        //     author: todo!(),
        //     date: todo!(),
        //     message: todo!(),
        // }
        Ok(commit)
    }
}

pub(crate) fn invoke() -> anyhow::Result<()> {
    // while commits have still parents contiune doing it
    // iteratively not recursively!

    let head_ref = std::fs::read_to_string(".git/HEAD").context("trying to read HEAD reference")?;
    let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
        anyhow::bail!("no commit onto detached HEAD");
    };
    let head_ref = head_ref.trim();

    let parent_hash = std::fs::read_to_string(format!(".git/{head_ref}"))
        .with_context(|| format!("read HEAD reference '{head_ref}'"));

    let mut parent_hash = match &parent_hash {
        Ok(p) => Some(p.trim()),
        Err(_) => {
            println!("fatal: your current HEAD does not have any commits yet");
            return Ok(());
        }
    };

    while parent_hash.is_some() {
        let current_hash = parent_hash.unwrap();
        let mut obj = Object::read(current_hash).context("creating obj from hash")?;

        let commit = Commit::from_obj(obj);

        // println!("commit {}", current_hash);
        // println!("Author: {}", current_hash);
        // println!("Date: {}\n", current_hash);
        // println!("{}", current_hash);

        // message
        // println!("commit obj size: {}", obj.expected_size);
        // println!("commit obj kind: {}", obj.kind);
        // let mut buffer = String::new();
        // obj.reader.read_to_string(& mut buffer);
        // println!("commit obj contetn: {}", buffer);

        parent_hash = None;
    }

    Ok(())
}
