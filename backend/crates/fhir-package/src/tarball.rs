use crate::{Manifest, Package, PackageError, PackageFile};
use flate2::read::GzDecoder;
use std::{
    io::{self, Read},
    path::{Component, Path},
};

/// Bounds on what reading one package may cost. A tarball is untrusted
/// input, and a small gzip can expand to far more than it weighs.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// The most bytes the tarball may expand to, headers included.
    pub max_uncompressed_bytes: u64,
    /// The most entries (files, folders) it may hold.
    pub max_files: usize,
}

/// Reads at most `remaining` bytes from `inner`, then fails, so an archive
/// that expands past the limit is refused rather than read to the end.
struct Bounded<R> {
    inner: R,
    remaining: u64,
    exceeded: bool,
}

impl<R: Read> Read for Bounded<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            // Anything more is over the limit; a clean end reads nothing.
            let mut probe = [0u8; 1];
            if self.inner.read(&mut probe)? == 0 {
                return Ok(0);
            }
            self.exceeded = true;
            return Err(io::Error::other("package exceeds the size limit"));
        }
        let max = usize::try_from(self.remaining)
            .unwrap_or(usize::MAX)
            .min(buf.len());
        let read = self.inner.read(&mut buf[..max])?;
        self.remaining -= read as u64;
        Ok(read)
    }
}

/// A path inside the archive as plain components: `./package/x.json` and
/// `package/x.json` alike. `None` for a path that climbs out (`..`) or is
/// absolute, which no package needs.
fn components(path: &Path) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(parts)
}

/// A package name without an npm scope: a FHIR package republished on npm
/// as `@scope/hl7.fhir.us.core` (as this repository's artifact packages
/// are) is installed as `hl7.fhir.us.core`.
fn unscoped(name: &str) -> &str {
    let name = name.trim();
    match name
        .strip_prefix('@')
        .and_then(|scoped| scoped.split_once('/'))
    {
        Some((_, name)) => name,
        None => name,
    }
}

/// Reads a package tarball, gzipped (`.tgz`, as registries serve them) or
/// not.
///
/// The package folder is wherever `package.json` sits nearest the top, so a
/// tarball made from any folder works, not only npm's `package/`. Only the
/// JSON files directly in that folder and in its example folder are kept.
///
/// # Errors
///
/// When the archive is unreadable, breaks `limits`, or has no
/// `package.json`.
pub fn read(bytes: &[u8], limits: &Limits) -> Result<Package, PackageError> {
    let gzipped = bytes.starts_with(&[0x1f, 0x8b]);
    let decompressed: Box<dyn Read + '_> = if gzipped {
        Box::new(GzDecoder::new(bytes))
    } else {
        Box::new(bytes)
    };
    let mut bounded = Bounded {
        inner: decompressed,
        remaining: limits.max_uncompressed_bytes,
        exceeded: false,
    };

    let json_files = {
        let mut archive = tar::Archive::new(&mut bounded);
        collect_json_files(&mut archive, limits)
    };
    let json_files = match json_files {
        Ok(files) => files,
        Err(_) if bounded.exceeded => {
            return Err(PackageError::TooLarge(limits.max_uncompressed_bytes));
        }
        Err(error) => return Err(error),
    };

    // The package folder: package.json's, nearest the top.
    let root = json_files
        .iter()
        .filter(|(parts, _)| parts.last().is_some_and(|name| name == "package.json"))
        .map(|(parts, _)| parts[..parts.len() - 1].to_vec())
        .min_by_key(Vec::len)
        .ok_or(PackageError::MissingManifest)?;

    let manifest_path = [root.as_slice(), &["package.json".to_string()]].concat();
    let mut manifest = json_files
        .iter()
        .find(|(parts, _)| *parts == manifest_path)
        .map(|(_, content)| serde_json::from_slice::<Manifest>(content))
        .ok_or(PackageError::MissingManifest)?
        .map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
    manifest.name = unscoped(&manifest.name).to_string();

    let example_folder = manifest
        .directories
        .example
        .clone()
        .unwrap_or_else(|| "example".to_string());
    let example_folder = components(Path::new(&example_folder)).unwrap_or_default();
    let example_root = [root.as_slice(), example_folder.as_slice()].concat();

    let mut resources = Vec::new();
    let mut examples = Vec::new();
    for (parts, content) in json_files {
        let Some((name, folder)) = parts.split_last() else {
            continue;
        };
        if folder == root.as_slice() {
            if name == "package.json" || name.starts_with('.') {
                continue;
            }
            resources.push(PackageFile {
                path: name.clone(),
                content,
            });
        } else if !example_folder.is_empty() && folder == example_root.as_slice() {
            if name.starts_with('.') {
                continue;
            }
            examples.push(PackageFile {
                path: [example_folder.as_slice(), std::slice::from_ref(name)]
                    .concat()
                    .join("/"),
                content,
            });
        }
    }

    Ok(Package {
        manifest,
        resources,
        examples,
    })
}

/// A file's path as components, and its content.
type ArchiveFile = (Vec<String>, Vec<u8>);

/// Every regular `.json` file in the archive no deeper than a package's
/// example folder, as path components and content. Deeper files (`other/`,
/// `openapi/`, ...) are skipped unread.
fn collect_json_files<R: Read>(
    archive: &mut tar::Archive<R>,
    limits: &Limits,
) -> Result<Vec<ArchiveFile>, PackageError> {
    // A `package/example/x.json` path, under one extra top folder at most.
    const MAX_DEPTH: usize = 4;

    let unreadable = |e: io::Error| PackageError::Unreadable(e.to_string());
    let mut files = Vec::new();
    for (count, entry) in archive.entries().map_err(unreadable)?.enumerate() {
        if count >= limits.max_files {
            return Err(PackageError::TooManyFiles(limits.max_files));
        }
        let mut entry = entry.map_err(unreadable)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let Some(parts) = components(&entry.path().map_err(unreadable)?) else {
            continue;
        };
        if parts.len() > MAX_DEPTH
            || !parts
                .last()
                .is_some_and(|name| name.to_ascii_lowercase().ends_with(".json"))
        {
            continue;
        }
        let mut content = Vec::with_capacity(
            usize::try_from(entry.header().size().unwrap_or(0))
                .unwrap_or(0)
                .min(1 << 20),
        );
        entry.read_to_end(&mut content).map_err(unreadable)?;
        files.push((parts, content));
    }
    Ok(files)
}

/// Packs a package folder (one holding `package.json`) into a gzipped
/// tarball laid out as npm packs one, under `package/`. Only what [`read`]
/// uses is packed: the JSON files directly in the folder and in its example
/// folder.
///
/// # Errors
///
/// When the folder has no readable `package.json` or a file cannot be read.
pub fn pack(folder: &Path) -> Result<Vec<u8>, PackageError> {
    let unreadable = |e: io::Error| PackageError::Unreadable(e.to_string());
    let manifest = std::fs::read(folder.join("package.json")).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => PackageError::MissingManifest,
        _ => unreadable(e),
    })?;
    let manifest = serde_json::from_slice::<Manifest>(&manifest)
        .map_err(|e| PackageError::InvalidManifest(e.to_string()))?;
    let example_folder = manifest
        .directories
        .example
        .unwrap_or_else(|| "example".to_string());

    let json_files = |relative: &Path| -> Result<Vec<(String, std::path::PathBuf)>, PackageError> {
        let directory = folder.join(relative);
        if !directory.is_dir() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&directory).map_err(unreadable)? {
            let entry = entry.map_err(unreadable)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type().map_err(unreadable)?.is_file()
                && name.to_ascii_lowercase().ends_with(".json")
            {
                let packed = Path::new("package").join(relative).join(&name);
                files.push((packed.to_string_lossy().replace('\\', "/"), entry.path()));
            }
        }
        files.sort();
        Ok(files)
    };
    let mut files = json_files(Path::new(""))?;
    files.extend(json_files(Path::new(&example_folder))?);

    let mut builder = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::default(),
    ));
    for (packed, path) in files {
        let content = std::fs::read(&path).map_err(unreadable)?;
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        builder
            .append_data(&mut header, packed, content.as_slice())
            .map_err(unreadable)?;
    }
    builder
        .into_inner()
        .and_then(flate2::write::GzEncoder::finish)
        .map_err(unreadable)
}
