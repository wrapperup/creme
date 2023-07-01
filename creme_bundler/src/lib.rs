use hex::ToHex;
use lightningcss::{
    stylesheet::{ParserFlags, ParserOptions},
    targets::Browsers,
};
use mime::Mime;
use once_cell::sync::Lazy;
use path_absolutize::Absolutize;
use serde::Serialize;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs::{self, File},
    io::{self, BufWriter},
    path::{Path, PathBuf},
    sync::Mutex,
};
use thiserror::Error;

mod css;

const MANIFEST_FILE: &str = "creme-manifest.json";

#[derive(Debug, Serialize)]
struct Manifest {
    assets: HashMap<String, String>,
}

static MANIFEST: Lazy<Mutex<Manifest>> = Lazy::new(|| {
    Mutex::new(Manifest {
        assets: HashMap::new(),
    })
});

#[derive(Debug, PartialEq, Eq)]
enum AssetType {
    Css,
    Other(Mime),
}

impl From<Mime> for AssetType {
    fn from(mime: Mime) -> Self {
        match mime.type_() {
            mime::TEXT => match mime.subtype() {
                mime::CSS => AssetType::Css,
                _ => AssetType::Other(mime),
            },
            _ => AssetType::Other(mime),
        }
    }
}

impl From<AssetType> for Mime {
    fn from(asset_type: AssetType) -> Self {
        match asset_type {
            AssetType::Css => mime::TEXT_CSS,
            AssetType::Other(mime) => mime,
        }
    }
}

#[derive(Debug)]
struct Asset {
    pub path: PathBuf,
    pub asset_type: AssetType,
}

#[derive(Debug)]
struct AssetSourceConfig {
    pub ignore_leading: Option<String>,
}

impl Default for AssetSourceConfig {
    fn default() -> Self {
        Self {
            ignore_leading: Some("_".to_string()),
        }
    }
}

#[derive(Debug)]
struct AssetSource {
    pub src_dir: PathBuf,
    pub sources: Vec<Asset>,
    pub css_sources: Vec<Asset>,
    pub _source_config: AssetSourceConfig,
}

impl AssetSource {
    pub fn from_asset_dir(src_dir: impl Into<PathBuf>) -> io::Result<Self> {
        let src_dir = src_dir.into();

        let mut sources = Vec::new();
        let mut css_sources = Vec::new();
        let source_config = AssetSourceConfig::default();

        Self::add_assets(
            &mut sources,
            &mut css_sources,
            &source_config.ignore_leading,
            &src_dir,
        )?;

        Ok(Self {
            src_dir,
            sources,
            css_sources,
            _source_config: source_config,
        })
    }

    /// Add an asset to the list of assets to be bundled.
    fn add_asset(
        assets: &mut Vec<Asset>,
        css_assets: &mut Vec<Asset>,
        ignore_leading: &Option<String>,
        path: impl Into<PathBuf>,
    ) {
        let path: PathBuf = path.into();

        if let Some(leading) = ignore_leading {
            if path
                .file_name()
                .unwrap() // this is always a file
                .to_string_lossy()
                .starts_with(leading)
            {
                return;
            }
        }

        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let asset_type = AssetType::from(mime);

        if asset_type == AssetType::Css {
            css_assets.push(Asset { path, asset_type });
        } else {
            assets.push(Asset { path, asset_type });
        }
    }

    /// Add all assets in a directory to the bundle.
    fn add_assets(
        assets: &mut Vec<Asset>,
        css_assets: &mut Vec<Asset>,
        ignore_leading: &Option<String>,
        path: impl Into<PathBuf>,
    ) -> io::Result<()> {
        let path = path.into();
        let dir = fs::read_dir(&path)?;

        for entry in dir.flatten() {
            let path = entry.path();

            // Recurse if directory
            if path.is_dir() {
                Self::add_assets(assets, css_assets, ignore_leading, path)?;
            } else {
                Self::add_asset(assets, css_assets, ignore_leading, path);
            }
        }

        Ok(())
    }
}

#[derive(Default, Debug)]
enum ReleaseMode {
    /// The file directory structure is preserved.
    /// Assets are served directly from the source directory.
    /// Files are also required to be hashed to prevent collisions.
    #[default]
    Development,

    /// The file directory structure is flattened.
    /// Files are optionally hashed for cache busting.
    Release { hashed: bool, flatten: bool },
}

/// The main struct for the library.
/// This is used to configure the library, and builds a `CremeBundler`.
#[derive(Debug, Default)]
pub struct Creme {
    /// The path to the public directory in the project.
    /// This is copied to the dist directory.
    public_dir: Option<PathBuf>,

    /// Contains the source directory and the assets to be processed.
    assets: Option<AssetSource>,

    /// Path to write the assets to, relative to the out public directory.
    /// Typically, this would be your build's `out_dir/public/assets` folder.
    out_assets_dir: Option<PathBuf>,

    /// The path to write the output to, relative to the out directory.
    /// Typically, this would be your build's `out_dir/public` folder.
    out_public_dir: Option<PathBuf>,

    /// The path to where all the generated files are written to.
    out_dir: Option<PathBuf>,

    /// How assets are written to the filesystem.
    release_mode: ReleaseMode,
}

impl Creme {
    /// Creates a new Creme instance.
    pub fn new() -> Self {
        Self {
            public_dir: None,
            assets: None,
            out_assets_dir: None,
            out_public_dir: None,
            out_dir: None,
            release_mode: ReleaseMode::default(),
        }
    }

    /// Creates a new Creme instance with the recommended defaults.
    /// The recommended defaults are:
    /// - `public` directory is copied to `dist` directory.
    /// - Assets are written to `assets` directory, inside the `dist` directory.
    /// - The `dist` directory is created inside the `out` directory.
    /// - The `out` directory is the `OUT_DIR` env var set by Cargo.
    ///
    /// # Errors
    ///
    /// This will return an error if the assets directory doesn't exist.
    pub fn recommended(self) -> CremeResult<Self> {
        self.detect_release_mode().default_config()
    }

    /// Detects the release mode based on the `debug_assertions` flag.
    pub fn detect_release_mode(self) -> Self {
        if cfg!(debug_assertions) {
            self.development()
        } else {
            self.release()
        }
    }

    /// Sets the bundler with the recommended defaults.
    ///
    /// # Errors
    ///
    /// This will return an error if the assets directory doesn't exist.
    pub fn default_config(self) -> CremeResult<Self> {
        Ok(self
            .set_public_dir("public")
            .set_assets_dir("assets")?
            .set_out_public_dir("public")
            .set_out_assets_dir("assets"))
    }

    /// Sets the release mode to release.
    pub fn release(self) -> Self {
        Self {
            release_mode: ReleaseMode::Release {
                hashed: true,
                flatten: true,
            },
            ..self
        }
    }

    /// Sets the release mode to development.
    pub fn development(self) -> Self {
        Self {
            release_mode: ReleaseMode::Development,
            ..self
        }
    }

    /// Sets the public directory.
    /// The public directory is copied to the dist directory.
    /// The default public directory is `public`.
    pub fn set_public_dir(self, public_dir: impl Into<PathBuf>) -> Self {
        Self {
            public_dir: Some(public_dir.into()),
            ..self
        }
    }

    /// Sets the directory to write the assets to.
    /// The default assets directory is `assets`.
    pub fn set_out_assets_dir(self, out_assets_dir: impl Into<PathBuf>) -> Self {
        Self {
            out_assets_dir: Some(out_assets_dir.into()),
            ..self
        }
    }

    /// Sets the directory to write the dist to.
    /// The default output directory is `dist`.
    pub fn set_out_public_dir(self, out_public_dir: impl Into<PathBuf>) -> Self {
        Self {
            out_public_dir: Some(out_public_dir.into()),
            ..self
        }
    }

    /// Sets the directory to write the output to.
    /// The default output directory is the `OUT_DIR` env var set by Cargo.
    pub fn out_dir(self, out_dir: impl Into<PathBuf>) -> Self {
        let out_dir: PathBuf = out_dir.into();
        let out_dir = out_dir.absolutize().unwrap().to_path_buf();

        Self {
            out_dir: Some(out_dir),
            ..self
        }
    }

    /// Sets the output directory to the `OUT_DIR` env var set by Cargo.
    /// This is useful when you want to embed the assets in the binary,
    /// and putting the output files into a directory that won't litter
    /// the project directory.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `OUT_DIR` environment variable is not set.
    pub fn out_dir_build_rs(self) -> CremeResult<Self> {
        Ok(Self {
            out_dir: Some(PathBuf::from(std::env::var("OUT_DIR")?)),
            ..self
        })
    }

    pub fn set_assets_dir(self, assets_dir: impl Into<PathBuf>) -> CremeResult<Self> {
        Ok(Self {
            assets: Some(AssetSource::from_asset_dir(assets_dir)?),
            ..self
        })
    }

    pub fn build(self) -> CremeResult<CremeBundler> {
        let Creme {
            public_dir,
            assets,
            out_assets_dir,
            out_public_dir,
            out_dir,
            release_mode,
        } = self;

        let assets = assets.unwrap();
        let out_public_dir = out_public_dir.unwrap();
        let out_assets_dir = out_assets_dir.unwrap();
        let public_dir = public_dir.unwrap();
        let out_dir = out_dir.unwrap();

        if std::env::var("OUT_DIR").is_ok() {
            match release_mode {
                ReleaseMode::Release {
                    hashed: _,
                    flatten: _,
                } => {
                    println!("cargo:rerun-if-changed={}", assets.src_dir.display());
                    println!("cargo:rerun-if-changed={}", public_dir.display());
                    println!(
                        "cargo:rustc-env=CREME_PUBLIC_DIR={}",
                        out_dir.join(&out_public_dir).display()
                    );
                    println!(
                        "cargo:rustc-env=CREME_ASSETS_DIR={}",
                        out_dir.join(&out_public_dir).join(&out_assets_dir).display()
                    );
                    println!(
                        "cargo:rustc-env=CREME_MANIFEST={}",
                        out_dir.join("creme-manifest.json").display()
                    );
                    println!("cargo:rustc-env=CREME_RELEASE_MODE=release");
                }
                ReleaseMode::Development => {
                    let base_dir = std::env::current_dir()?;
                    println!(
                        "cargo:rustc-env=CREME_PUBLIC_DIR={}",
                        base_dir.join(&public_dir).display()
                    );
                    println!(
                        "cargo:rustc-env=CREME_ASSETS_DIR={}",
                        base_dir.join(&assets.src_dir).display()
                    );
                    println!("cargo:rustc-env=CREME_RELEASE_MODE=development");
                }
            };
        }

        Ok(CremeBundler {
            public_dir,
            assets,
            out_assets_dir,
            out_public_dir,
            out_dir,
            release_mode,
        })
    }

    /// Bundles the assets. Shortcut for `self.build()?.bundle()`.
    pub fn bundle(self) -> CremeResult<()> {
        self.build()?.bundle()
    }
}

pub struct CremeBundler {
    /// The path to the public directory in the project.
    /// This is copied to the dist directory.
    public_dir: PathBuf,

    /// Contains the source directory and the assets to be processed.
    assets: AssetSource,

    /// Path to write the assets to, relative to the dist directory.
    /// Typically, this would be your build's `out_dir/dist/assets` folder.
    out_assets_dir: PathBuf,

    /// The path to write the output to, relative to the out directory.
    /// Typically, this would be your build's `out_dir/dist` folder.
    out_public_dir: PathBuf,

    /// The path to where all the generated files are written to.
    out_dir: PathBuf,

    /// How should the output be written to the filesystem.
    release_mode: ReleaseMode,
}

impl CremeBundler {
    fn filename_with_hash(filename: &OsStr, content: &[u8]) -> OsString {
        let path = Path::new(filename);

        let mut digest = [0; 4];
        blake3::Hasher::new()
            .update(content)
            .finalize_xof()
            .fill(&mut digest);

        let digest = digest.encode_hex::<String>();

        let filename = path.file_stem().unwrap();
        let ext = path.extension();

        if let Some(ext) = ext {
            let mut hashed_path =
                OsString::with_capacity(filename.len() + ext.len() + 1 + digest.len());
            hashed_path.push(filename);
            hashed_path.push("-");
            hashed_path.push(digest);
            hashed_path.push(".");
            hashed_path.push(ext);
            hashed_path
        } else {
            let mut hashed_path = OsString::with_capacity(filename.len() + 1 + digest.len());
            hashed_path.push(filename);
            hashed_path.push("-");
            hashed_path.push(digest);
            hashed_path
        }
    }

    fn process_asset(
        asset: &Asset,
        out_dir: &Path,
        assets_dir: &PathBuf,
        _flatten: bool,
        hashed: bool,
    ) -> CremeResult<()> {
        let Asset { path, asset_type } = asset;

        let content = Self::process_file(path, assets_dir, asset_type)?;

        let filename = path.file_name().unwrap();
        let filename = if hashed {
            Self::filename_with_hash(filename, &content)
        } else {
            filename.to_owned()
        };

        let asset_file_path = assets_dir.join(filename);

        {
            let out_file_path = out_dir.join(&asset_file_path);
            fs::write(out_file_path, content)?;
        }

        let src_path = path.strip_prefix(assets_dir).unwrap();

        let src_url = src_path.to_str().unwrap().replace('\\', "/");
        let dest_url = asset_file_path.to_str().unwrap().replace('\\', "/");

        MANIFEST.lock().unwrap().assets.insert(src_url, dest_url);

        Ok(())
    }

    fn process_file(
        path: impl Into<PathBuf>,
        assets_dir: &PathBuf,
        asset_type: &AssetType,
    ) -> CremeResult<Vec<u8>> {
        let path: PathBuf = path.into();
        Ok(match asset_type {
            AssetType::Css => {
                // TODO: config, maybe modularize this?
                // Also lots of copying here.
                let parser_options = ParserOptions {
                    flags: ParserFlags::NESTING | ParserFlags::CUSTOM_MEDIA,
                    ..Default::default()
                };

                let targets = Browsers::from_browserslist([">= 0.25%"])
                    .map_err(|e| CremeError::Css(css::BundleError::Browsers(e)))?;

                css::process_css(&path, parser_options, targets, assets_dir).into_bytes()
            }
            _ => fs::read(&path)?,
        })
    }

    fn copy_recursively(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let filetype = entry.file_type()?;
            if filetype.is_dir() {
                Self::copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    pub fn bundle(&self) -> CremeResult<()> {
        let CremeBundler {
            public_dir,
            assets,
            out_assets_dir,
            out_public_dir,
            out_dir,
            release_mode,
            ..
        } = self;

        if let ReleaseMode::Release { flatten, hashed } = release_mode {
            let dist_dir = out_dir.join(out_public_dir);

            // Remove dist directory if it exists
            if out_dir.exists() {
                fs::remove_dir_all(out_dir)?;
            }

            // Create assets directory
            fs::create_dir_all(&dist_dir.join(out_assets_dir))?;

            // Copy public assets
            Self::copy_recursively(public_dir, &dist_dir)?;

            // Process assets
            for asset in &assets.sources {
                Self::process_asset(asset, &dist_dir, out_assets_dir, *flatten, *hashed)?;
            }

            // Process CSS assets
            for asset in &assets.css_sources {
                Self::process_asset(asset, &dist_dir, out_assets_dir, *flatten, *hashed)?;
            }

            let file = File::create(out_dir.join(MANIFEST_FILE))?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &*MANIFEST)?;
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum CremeError {
    #[error("asset dir error: {0}")]
    AssetsDirDoesNotExist(PathBuf),

    #[error("public path error: {0}")]
    PublicDirDoesNotExist(PathBuf),

    #[error("out assets path error: {0}")]
    AssetsOutDirMustBeRelative(PathBuf),

    #[error("out dist path error: {0}")]
    DistOutDirMustBeRelative(PathBuf),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path error: {0}")]
    NotAFile(PathBuf),

    #[error("path error: {0}")]
    InvalidFileName(PathBuf),

    #[error("env error: {0}")]
    Env(#[from] std::env::VarError),

    #[error("css error: {0}")]
    Css(#[from] css::BundleError),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type CremeResult<T> = std::result::Result<T, CremeError>;
