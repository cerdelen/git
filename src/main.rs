#[allow(unused_imports)]
use std::env;
use std::{fs, path::PathBuf};

pub(crate) mod commands;
pub(crate) mod objects;
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
        object_hash: String
    },
    HashObject {
        #[clap(short = 'w')]
        save: bool,
        file: PathBuf
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        },
        Command::CatFile { pretty_print, object_hash }
            => commands::cat_file::invoke(pretty_print, &object_hash)?,
        Command::HashObject { save, file }
            => commands::hash_object::invoke(save, &file)?,
        Command::LsTree { name_only, tree_hash }
            => commands::ls_tree::invoke(name_only, &tree_hash)?,
    }
    Ok(())
}
