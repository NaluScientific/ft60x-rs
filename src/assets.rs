// The RustEmbed proc macro generates warnings about non-upper case globals.
#![allow(non_upper_case_globals)]

use std::{fs::File, io::Write, path::Path};

use libloading::Library;
use once_cell::sync::OnceCell;
use rust_embed::RustEmbed;
use tempfile::TempDir;

use crate::{D3xxError, Result};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

/// The dynamic library which all D3xx functions will be loaded from.
static LIBRARY: OnceCell<Library> = OnceCell::new();

#[cfg(target_os = "windows")]
const LIBRARY_NAME: &'static str = "FTD3XX.dll";

#[cfg(target_os = "linux")]
const LIBRARY_NAME: &'static str = "libftd3xx.so";

/// Load the dynamic library at the given path.
///
/// # Errors
/// - [`D3xxError::LibraryAlreadyLoaded`] if the library has already been loaded.
/// - [`D3xxError::LibraryLoadFailed`] if the library could not be loaded.
pub fn load_dylib(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    match LIBRARY.get() {
        Some(_) => Err(D3xxError::LibraryAlreadyLoaded),
        None => {
            LIBRARY.get_or_try_init(|| {
                unsafe { Library::new(path) }
            })?;
            Ok(())
        }
    }
}

/// Load the bundled dynamic library for this platform.
///
/// The library will be read from the bundled assets and written to a temporary
/// directory before being loaded. This is necessary as dynamic libraries
/// cannot be loaded directly from memory.
///
/// # Errors
/// - [`D3xxError::LibraryAlreadyLoaded`] if the library has already been loaded.
/// - [`D3xxError::LibraryLoadFailed`] if the library could not be loaded.
pub fn load_bundled_dylib() -> Result<()> {
    static TEMP_DIR: OnceCell<TempDir> = OnceCell::new();

    if LIBRARY.get().is_some() {
        return Err(D3xxError::LibraryAlreadyLoaded);
    }

    let dylib_path = TEMP_DIR
        .get_or_try_init(tempfile::tempdir)?
        .path()
        .join(LIBRARY_NAME);
    let asset = Assets::get(LIBRARY_NAME).expect("library asset not found");
    let _ = File::create(&dylib_path)?
        .write_all(asset.data.as_ref())?;
    load_dylib(dylib_path)
}

/// Fetches the dynamic library.
///
/// # Errors
/// - [`D3xxError::LibraryNotLoaded`] if the library has not been loaded.
pub(crate) fn d3xx_lib() -> Result<&'static Library> {
    LIBRARY.get().ok_or(D3xxError::LibraryNotLoaded)
}
