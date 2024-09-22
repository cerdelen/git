#[allow(unused_imports)]
use std::env;
use std::{fs, path::PathBuf};

pub(crate) mod commands;
pub(crate) mod objects;
use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        save: bool,
        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        tree_hash: String,
        #[clap(short = 'p')]
        parent_hash: Option<String>,
        #[clap(short = 'm')]
        commit_message: String,
    },
    Commit {
        #[clap(short = 'm')]
        commit_message: String,
    },
    Log,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            if std::fs::exists(".git").context("trying to check if .git exists")? {
                println!("Already Initialized git directory");
                return Ok(());
            }
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)?,
        Command::HashObject { save, file } => commands::hash_object::invoke(save, &file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::WriteTree => commands::write_tree::invoke()?,
        Command::CommitTree {
            parent_hash,
            commit_message,
            tree_hash,
        } => commands::commit_tree::invoke(parent_hash, commit_message, tree_hash)?,
        Command::Commit { commit_message } => {
            let head_ref =
                std::fs::read_to_string(".git/HEAD").context("trying to read HEAD reference")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("no commit onto detached HEAD");
            };
            let head_ref = head_ref.trim();

            let parent_hash = std::fs::read_to_string(format!(".git/{head_ref}"))
                .with_context(|| format!("read HEAD reference '{head_ref}'"));

            let tree = commands::write_tree::recursive_tree_write(std::path::Path::new("."))
                .context("making tree object")?;

            let parent_hash = match &parent_hash {
                Ok(p) => Some(p.trim()),
                Err(_) => None,
            };

            let commit = commands::commit_tree::write_commit(
                &commit_message,
                &hex::encode(tree),
                parent_hash,
            )
            .context("create a real commit")?;

            let commit = hex::encode(commit);

            let p = format!(".git/{head_ref}");
            let path = std::path::Path::new(&p);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).context("create parent folders of HEAD reference")?;
            }

            std::fs::write(format!(".git/{head_ref}"), &commit)
                .with_context(|| format!("updating HEAD reference {head_ref}"))?;

            println!("new HEAD ref is now {}", &commit);
        }
        Command::Log => commands::log::invoke()?,
    }
    Ok(())
}
