use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;
use std::ffi::CStr;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Cursor;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use crate::commands::write_tree::TreeEntry;

pub(crate) struct GitHash {
    hash: [u8; 20],
}
impl fmt::Display for GitHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.hash))
    }
}

impl GitHash {
    fn from_str(hash: &str) -> anyhow::Result<Self> {
        let mut buf = [0;20];
        hex::decode_to_slice(hash, & mut buf).context("trying to create hash from str slice")?;
        Ok(GitHash {
            hash: buf,
        })
    }
    fn from_slice(hash: &[u8; 20]) -> anyhow::Result<Self> {
        Ok(GitHash {
            hash: hash.clone(),
        })
    }
    fn to_str(&self) -> String {
        format!("{}", self)
    }
    fn hash_start(&self) -> [u8; 2] {
        self.hash[0..2].try_into().expect("i make a slice of size 2 from a slice of size 20 but it fails?")
    }
    fn hash_end(&self) -> [u8; 2] {
        self.hash[2..20].try_into().expect("i make a slice of size 2 from a slice of size 20 but it fails?")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn commit_obj(
        tree_hash: &str,
        author: &str,
        email: &str,
        commit_message: &str,
        parent: Option<&str>,
    ) -> anyhow::Result<Object<impl Read>> {
        use std::fmt::Write;
        let mut commit = String::new();

        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .context("current system time is before UNIX epoch")?;

        writeln!(commit, "tree {tree_hash}")?;

        if let Some(parent_hash) = parent {
            writeln!(commit, "parent {parent_hash}")?;
        }

        // author line
        writeln!(commit, "author {author} <{email}> {} +0000", time.as_secs())?;

        // commiter line
        writeln!(
            commit,
            "commiter {author} <{email}> {} +0000",
            time.as_secs()
        )?;

        // empty line
        writeln!(commit, "")?;

        // commit message
        writeln!(commit, "{commit_message}")?;

        Ok(Object {
            kind: Kind::Commit,
            expected_size: commit.len() as u64,
            reader: Cursor::new(commit),
        })
    }

    pub(crate) fn tree_obj_from_vec(
        mut entries: Vec<TreeEntry>,
    ) -> anyhow::Result<Object<impl Read>> {
        // sort tree by alphabetical order of name(?)
        entries.sort_by(|a, b| {
            let mut name_a = a.name.clone();
            let mut name_b = b.name.clone();
            if a.kind == Kind::Tree {
                name_a.push("/");
            }
            if b.kind == Kind::Tree {
                name_b.push("/");
            }
            name_a.cmp(&name_b)
        });
        let mut buffer: Vec<u8> = Vec::new();
        for entry in entries {
            buffer.extend(format!("{:06o} ", entry.mode).as_bytes());
            buffer.extend(entry.name.clone().into_vec());
            buffer.extend(format!("\0").as_bytes());
            buffer.extend(entry.hash);
        }

        Ok(Object {
            kind: Kind::Tree,
            expected_size: buffer.len() as u64,
            reader: Cursor::new(buffer),
        })
    }

    pub(crate) fn blob_from_file(path: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = path.as_ref();
        let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;
        // ToDo: there is a race condition if file gets changed inbetween
        let file = std::fs::File::open(file).with_context(|| format!("open {}", file.display()))?;

        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }

    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let file = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(file);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("read header from .git/objects")?;
        let header = CStr::from_bytes_with_nul(&buf).expect("reading headers nul");
        let header = header
            .to_str()
            .context(".git/objects file headers are not in UTF-8")?;

        let Some((kind, size)) = header.split_once(' ') else {
            anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
        };
        let kind = match kind {
            "blob" => Kind::Blob,
            "tree" => Kind::Tree,
            "commit" => Kind::Commit,
            _ => anyhow::bail!("reading .git/objects with unknown kind: '{kind}'"),
        };
        let size = size
            .parse::<u64>()
            .context(".git/objects file header has invalid size: {size}")?;
        let z = z.take(size);

        Ok(Object {
            kind,
            expected_size: size,
            reader: z,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    // create hash from obj
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<GitHash> {
        let writer = ZlibEncoder::new(writer, Compression::default());
        let mut writer = HashWriter {
            writer,
            hasher: Sha1::new(),
        };

        write!(writer, "{} {}\0", self.kind, self.expected_size)?;
        std::io::copy(&mut self.reader, &mut writer).context("copy hash into Object")?;
        let _ = writer.writer.finish();
        let hash = writer.hasher.finalize();

        Ok(GitHash::from_slice(&hash.into())?)
    }

    // make ob in .git/objects folder
    pub(crate) fn write_obj(self) -> anyhow::Result<GitHash> {
        let temp = "temporary";

        let hash = self
            .write(std::fs::File::create(temp).context("couldn't create temp for obj")?)
            .context("couldn't write to temp")?;

        // let hash_hex = hex::encode(hash);

        let _ = fs::create_dir(format!(".git/objects/{}", &hash.hash_end() &hash_hex[..2]))
            .context("create parent folder of first 2 bytes of hash");

        fs::rename(
            temp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("move obj file to right place")?;

        Ok(hash)
    }
}

struct HashWriter<W> {
    writer: W,
    hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
