use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook, FontInfo, Tag};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::downloader::SystemDownloader;
use typst_kit::files::{FileStore, FsRoot, SystemFiles};
use typst_kit::fonts::{FontSource, FontStore};
use typst_kit::packages::{FsPackages, SystemPackages, UniversePackages};

const USER_AGENT: &str = concat!("lgl/", env!("CARGO_PKG_VERSION"));

/// A font file bundled with lgl (see `fonts/`), compressed with DEFLATE.
pub struct BundledFont {
    /// The file's path below `fonts/`.
    pub name: &'static str,
    /// The FNV-1a hash of the uncompressed file.
    pub hash: u64,
    /// The size of the uncompressed file.
    pub size: usize,
    /// The compressed file.
    pub data: &'static [u8],
}

impl BundledFont {
    /// Decompresses the file.
    fn bytes(&self) -> Option<Bytes> {
        let data =
            miniz_oxide::inflate::decompress_to_vec_with_limit(self.data, self.size)
                .ok()?;
        Some(Bytes::new(data))
    }

    /// Describes every face in the file. This decompresses the file.
    fn describe(&self) -> Vec<(u32, FontInfo)> {
        let Some(data) = self.bytes() else { return Vec::new() };
        Font::iter(data)
            .map(|font| {
                let mut info = font.info().clone();
                drop_optical_size(&mut info);
                (font.index(), info)
            })
            .collect()
    }
}

/// Variable fonts with an optical-size axis often name their default instance
/// after its size, like "DM Sans 9pt". Typst picks the optical size from the
/// text size, so the family is known by its plain name.
fn drop_optical_size(info: &mut FontInfo) {
    if !info.axes.iter().any(|axis| axis.tag == Tag::from_bytes(b"opsz")) {
        return;
    }
    let Some((name, size)) = info.family.rsplit_once(' ') else { return };
    let is_size = size.strip_suffix("pt").is_some_and(|number| {
        !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.')
    });
    if is_size {
        info.family = name.to_string();
    }
}

include!(concat!(env!("OUT_DIR"), "/bundled_fonts.rs"));

/// The metadata of the bundled fonts, keyed by file hash, so that lgl does
/// not have to decompress every font to start. A file missing from the index
/// is described at startup instead. The `bundled_font_index_is_current` test
/// writes a fresh index when `fonts/` changes.
const FONT_INDEX: &str = include_str!("../../../fonts/index.json");

/// A face of a bundled font file, decompressed when a document first uses it.
struct BundledFace {
    file: &'static BundledFont,
    index: u32,
}

impl FontSource for BundledFace {
    fn load(&self) -> Option<Font> {
        Font::new(self.file.bytes()?, self.index)
    }
}

/// The fonts available to documents: Typst's embedded fonts, the families
/// bundled with lgl (see `fonts/`), and any fonts in `dirs`.
pub fn font_store(dirs: &[PathBuf]) -> FontStore {
    let mut fonts = FontStore::new();
    fonts.extend(typst_kit::fonts::embedded());
    let mut index: HashMap<String, Vec<(u32, FontInfo)>> =
        serde_json::from_str(FONT_INDEX).unwrap_or_default();
    for file in BUNDLED_FONTS {
        let faces = index
            .remove(&format!("{:016x}", file.hash))
            .unwrap_or_else(|| file.describe());
        for (face, info) in faces {
            fonts.push((BundledFace { file, index: face }, info));
        }
    }
    for dir in dirs {
        fonts.extend(typst_kit::fonts::scan(dir));
    }
    fonts
}

/// The font index for the bundled fonts, one file per line, in the format of
/// `fonts/index.json`.
pub fn bundled_font_index() -> String {
    let entries: BTreeMap<String, String> = BUNDLED_FONTS
        .iter()
        .map(|file| {
            let faces = serde_json::to_string(&file.describe()).unwrap();
            (format!("{:016x}", file.hash), faces)
        })
        .collect();
    let lines: Vec<String> = entries
        .iter()
        .map(|(hash, faces)| format!("\"{hash}\": {faces}"))
        .collect();
    format!("{{\n{}\n}}\n", lines.join(",\n"))
}

#[derive(Debug, Clone, Default)]
pub struct WorldOptions {
    pub root: Option<PathBuf>,
    pub font_dirs: Vec<PathBuf>,
    pub package_path: Option<PathBuf>,
    pub package_cache_path: Option<PathBuf>,
    pub creation_timestamp: Option<i64>,
}

pub struct LabWorld {
    root: PathBuf,
    main: FileId,
    library: LazyHash<Library>,
    fonts: FontStore,
    files: FileStore<SystemFiles>,
    now: Time,
}

impl LabWorld {
    pub fn new(input: &Path, options: &WorldOptions) -> Result<Self, WorldError> {
        let input = input.canonicalize().map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => WorldError::InputNotFound(input.to_path_buf()),
            _ => WorldError::Io(err),
        })?;

        let root = options
            .root
            .as_deref()
            .or_else(|| input.parent())
            .unwrap_or(Path::new("."));
        let root = root.canonicalize().map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => WorldError::RootNotFound(root.to_path_buf()),
            _ => WorldError::Io(err),
        })?;

        let vpath = VirtualPath::virtualize(&root, &input)
            .map_err(|_| WorldError::InputOutsideRoot)?;
        let main = RootedPath::new(VirtualRoot::Project, vpath).intern();

        let fonts = font_store(&options.font_dirs);

        let packages = SystemPackages::from_parts(
            options
                .package_path
                .clone()
                .map(FsPackages::new)
                .or_else(FsPackages::system_data),
            options
                .package_cache_path
                .clone()
                .map(FsPackages::new)
                .or_else(FsPackages::system_cache),
            UniversePackages::new(SystemDownloader::new(USER_AGENT)),
        );

        let now = match options.creation_timestamp {
            Some(timestamp) => Time::fixed_timestamp(timestamp)
                .map_err(|_| WorldError::InvalidTimestamp)?,
            None => Time::system(),
        };

        Ok(Self {
            files: FileStore::new(SystemFiles::new(FsRoot::new(root.clone()), packages)),
            root,
            main,
            library: LazyHash::new(
                Library::builder([typst_pdf::FORMAT]).with_probes(true).build(),
            ),
            fonts,
            now,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn reset(&mut self) {
        self.files.reset();
        self.now.reset();
    }

    pub fn display_name(&self, id: FileId) -> String {
        match id.root() {
            VirtualRoot::Project => id.vpath().get_without_slash().into(),
            VirtualRoot::Package(package) => {
                format!("{package}{}", id.vpath().get_with_slash())
            }
        }
    }
}

impl World for LabWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.now.today(offset)
    }
}

#[derive(Debug)]
pub enum WorldError {
    InputNotFound(PathBuf),
    RootNotFound(PathBuf),
    InputOutsideRoot,
    InvalidTimestamp,
    Io(io::Error),
}

impl fmt::Display for WorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputNotFound(path) => {
                write!(f, "input file not found: {}", path.display())
            }
            Self::RootNotFound(path) => {
                write!(f, "root directory not found: {}", path.display())
            }
            Self::InputOutsideRoot => {
                write!(f, "input file must be contained in the project root")
            }
            Self::InvalidTimestamp => write!(f, "creation timestamp out of range"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for WorldError {}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn bundled_font_index_is_current() {
        let fresh = super::bundled_font_index();
        let parse = |text: &str| serde_json::from_str::<serde_json::Value>(text).unwrap();
        if parse(&fresh) != parse(super::FONT_INDEX) {
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/font-index.json");
            std::fs::write(&path, fresh).unwrap();
            panic!("fonts/index.json is out of date; replace it with {}", path.display());
        }
    }

    #[test]
    fn bundled_fonts_load() {
        for file in super::BUNDLED_FONTS {
            assert!(!file.describe().is_empty(), "{} has no usable face", file.name);
        }
    }
}
