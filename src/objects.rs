use anyhow::Context;
use flate2::read::ZlibDecoder;
// use flate2::write::ZlibEncoder;
// use flate2::Compression;
// use sha1::Digest;
// use sha1::Sha1;
use std::ffi::CStr;
use std::fmt;
// use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
// use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "Blob"),
            Kind::Tree => write!(f, "Tree"),
            Kind::Commit => write!(f, "Commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}


impl Object<()> {
    // pub(crate) fn blob_from_file(path: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
    //     let file = path.as_ref();
    //     let stat = std::fs::metadata(file).with_context(|| format!("stat {}", file.display()))?;
    //     // ToDo: there is a race condition if file gets changed inbetween
    //     let file = std::fs::File::open(file).with_context(|| format!("open {}", file.display()))?;
    //
    //     Ok(Object{
    //         kind: Kind::Blob,
    //         expected_size: stat.len(),
    //         reader: file
    //     })
    // }

    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let file = std::fs::File::open(format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects")?;
        let z = ZlibDecoder::new(file);
        let mut z = BufReader::new(z);
        let mut buf = Vec::new();
        z.read_until(0, &mut buf)
            .context("read header from .git/objects")?;
        let header = CStr::from_bytes_with_nul(&buf)
            .expect("reading headers nul");
        let header = header
            .to_str()
            .context(".git/objects file headers are not in UTF-8")?;

        let Some((kind, size)) = header.split_once(' ')
            else {
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
