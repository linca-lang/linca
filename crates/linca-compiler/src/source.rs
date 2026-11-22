//! Source files of a schema root.

use std::io;
use std::path::Path;

const EXTENSION: &str = "linca";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path relative to the schema root, with `/` as separator.
    pub path: String,
    pub text: String,
}

/// The files of one schema, sorted by path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceSet {
    files: Vec<SourceFile>,
}

impl SourceSet {
    pub fn from_files(mut files: Vec<SourceFile>) -> Self {
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Self { files }
    }

    pub fn files(&self) -> &[SourceFile] {
        &self.files
    }

    /// A schema made of one file, named after the file.
    pub fn load_file(path: &Path) -> io::Result<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?;
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_files(vec![SourceFile {
            path: name.to_owned(),
            text,
        }]))
    }

    /// A schema root: every `.linca` file below `root`.
    pub fn load_root(root: &Path) -> io::Result<Self> {
        let mut files = Vec::new();
        collect(root, "", &mut files)?;
        Ok(Self::from_files(files))
    }
}

fn collect(dir: &Path, prefix: &str, out: &mut Vec<SourceFile>) -> io::Result<()> {
    let mut entries = std::fs::read_dir(dir)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().into_string().map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "file name is not valid UTF-8")
        })?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(&path, &format!("{prefix}{name}/"), out)?;
        } else if path.extension().is_some_and(|ext| ext == EXTENSION) {
            let text = std::fs::read_to_string(&path)?;
            out.push(SourceFile {
                path: format!("{prefix}{name}"),
                text,
            });
        }
    }
    Ok(())
}
