use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use typst::diag::FileResult;
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::datetime::Time;
use typst_kit::downloader::SystemDownloader;
use typst_kit::files::{FileStore, FsRoot, SystemFiles};
use typst_kit::fonts::FontStore;
use typst_kit::packages::{FsPackages, SystemPackages, UniversePackages};

const USER_AGENT: &str = concat!("lgl/", env!("CARGO_PKG_VERSION"));

include!(concat!(env!("OUT_DIR"), "/bundled_fonts.rs"));

/// The fonts available to documents: Typst's embedded fonts, the families
/// bundled with lgl (see `fonts/`), and any fonts in `dirs`.
pub fn font_store(dirs: &[PathBuf]) -> FontStore {
    let mut fonts = FontStore::new();
    fonts.extend(typst_kit::fonts::embedded());
    fonts.extend(BUNDLED_FONTS.iter().flat_map(|data| {
        Font::iter(Bytes::new(*data)).map(|font| {
            let info = font.info().clone();
            (font, info)
        })
    }));
    for dir in dirs {
        fonts.extend(typst_kit::fonts::scan(dir));
    }
    fonts
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
