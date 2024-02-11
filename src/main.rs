use anyhow::Context;
use clap::{Parser, Subcommand};
use iconv::decode;
use std::{fs::File, path::PathBuf};
use zip::{ZipArchive, ZipWriter};

/// A small utility to convert the file names in a zip file from one encoding to another.
///
/// The input encoding is specified as a string, which is passed to the iconv library.
/// The output encoding is always UTF-8.
#[derive(Parser)]
#[clap(name = "polyzip", version, about)]
struct Args {
    /// The encoding of the file names in the zip file.
    /// A list of supported encodings can be found at https://www.gnu.org/software/libiconv/
    #[arg(index = 1)]
    input_encoding: String,

    #[command(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List the converted filenames in the zip file
    List {
        /// A path to a zip file
        #[arg(index = 1)]
        file: PathBuf,
    },

    /// Convert the filenames in the zip file and write the result to a new file
    Convert {
        /// A path to a zip file
        #[arg(index = 1)]
        file: PathBuf,

        /// Where to put the new zip file
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Convert the filenames in the zip file in place
    ConvertInPlace {
        /// A path to a zip file
        #[arg(index = 1)]
        file: PathBuf,

        /// Whether to create a backup of the original file, with a .bak extension
        #[arg(short, long, default_value = "true")]
        create_backup: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let args: Args = Args::parse();

    match args.subcommand {
        Command::List { file } => {
            let zipfile = File::open(&file).context(format!("Failed to open {:?}", file))?;
            list_zip_contents(zipfile, &args.input_encoding)?;
        }
        Command::Convert { file, output } => {
            let zipfile = File::open(&file).context(format!("Failed to open {:?}", file))?;
            let new_zipfile =
                File::create(&output).context(format!("Failed to create {:?}", output))?;
            unzip(zipfile, new_zipfile, &args.input_encoding)?;
        }
        Command::ConvertInPlace {
            file,
            create_backup,
        } => {
            let backup_path = &file.with_extension(
                file.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_string()
                    + ".bak",
            );
            std::fs::rename(&file, backup_path)?;

            let zipfile = File::open(&file).context(format!("Failed to open {:?}", file))?;
            let new_zipfile =
                File::create(&file).context(format!("Failed to create {:?}", file))?;
            if let Err(e) = unzip(zipfile, new_zipfile, &args.input_encoding) {
                std::fs::rename(backup_path, file)?;
                anyhow::bail!(e)
            } else if !create_backup {
                std::fs::remove_file(backup_path)?;
            }
        }
    }

    Ok(())
}

fn list_zip_contents(zipfile: File, input_encoding: &str) -> anyhow::Result<()> {
    let mut zip = ZipArchive::new(zipfile)?;

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let name = file.name_raw();
        match decode(name, input_encoding) {
            Ok(converted_name) => {
                println!("{} -> {}", String::from_utf8_lossy(name), converted_name)
            }
            Err(e) => eprintln!("{}: {}", String::from_utf8_lossy(name), e),
        }
    }

    Ok(())
}

fn unzip(zipfile: File, new_zipfile: File, input_encoding: &str) -> anyhow::Result<()> {
    let mut zip = ZipArchive::new(zipfile)?;
    let mut builder = ZipWriter::new(new_zipfile);

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        let name = file.name_raw().to_vec();
        let converted_name = decode(&name, input_encoding)?;
        builder.raw_copy_file_rename(file, converted_name)?;
    }

    builder.finish()?;

    Ok(())
}
