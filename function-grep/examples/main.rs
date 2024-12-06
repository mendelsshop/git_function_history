#![deny(clippy::unwrap_used, clippy::expect_used)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(missing_debug_implementations, clippy::missing_panics_doc)]
#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
#![deny(clippy::use_self, rust_2018_idioms)]
use function_grep::supported_languages::{InstantiateMap, InstantiationError};
use function_grep::{supported_languages::predefined_languages, ParsedFile};

use clap::Parser;
use std::{fs::File, io::Read, path::PathBuf};
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// The file to search in.
    file: PathBuf,
    /// The function name you want to search for.
    name: String,
}

#[derive(Debug)]
pub enum Error {
    CouldNotOpenFile(std::io::Error),
    CouldNotReadFile(std::io::Error),
    LibraryError(function_grep::Error),
    InstatantiationError(InstantiationError),
}

///
/// # Errors
pub fn main() -> Result<(), Error> {
    // get the cli args
    let args = Args::parse();

    // open the file
    let mut file = File::open(&args.file).map_err(Error::CouldNotOpenFile)?;
    // read the file in
    let mut code = String::new();
    let languages = predefined_languages()
        .instantiate_map(&args.name)
        .map_err(Error::InstatantiationError)?;
    file.read_to_string(&mut code)
        .map_err(Error::CouldNotReadFile)?;
    let file_name = &args.file.to_string_lossy();
    // search the file for function with the given name
    let found = ParsedFile::search_file_with_name(&code, file_name, &languages)
        .map_err(Error::LibraryError)?;
    // and print the results
    println!("{found}");
    Ok(())
}
