use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::File;
use std::path::Path;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::pan_finder::config::Configuration;

#[derive(PartialEq, Debug)]
pub enum FileType {
    Unknown,

    Text,
}

/// Configuration of application
#[derive(Debug)]
pub struct FilesDescription {
    pub file_entry: DirEntry,
    pub file_type: FileType,
}

/// Get list of files to analyse
pub fn get_files_list(config: &Configuration) -> Vec<FilesDescription> {
    let mut files_list: Vec<FilesDescription> = Vec::new();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let progress_bar = if config.quiet_mode {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(10)
    };
    progress_bar.set_style(spinner_style.clone());
    progress_bar.set_prefix("Build list of files");

    for entry in WalkDir::new(&config.search_dir)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, &config.exclusions))
    {
        progress_bar.set_message("In progress...");
        progress_bar.inc(1);
        let ent = entry.unwrap();
        if ent.file_type().is_file() && !is_file_empty(ent.path()) {
            let file_type = detect_file_type(ent.path());

            if file_type != FileType::Unknown {
                files_list.push(FilesDescription {
                    file_entry: ent,
                    file_type,
                });
            }
        }
    }
    progress_bar.finish_with_message("Done!");

    files_list
}

/// Check is a file or a directory is excluded from analyse
fn is_excluded(entry: &DirEntry, exclusions: &Vec<String>) -> bool {
    entry
        .path()
        .to_str()
        .map(|s| {
            for exclusion in exclusions {
                if s.contains(exclusion) {
                    return true;
                }
            }
            false
        })
        .unwrap_or(false)
}

/// Read the first n bytes of a file
fn read_up_to(file: &mut impl std::io::Read, mut buf: &mut [u8]) -> Result<usize, std::io::Error> {
    let buf_len = buf.len();

    while !buf.is_empty() {
        match file.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(buf_len - buf.len())
}

/// Check is a file is a text one
///
/// _Note:_ a file is considered be a text one if there is no `0` in its first 8000 bytes
fn is_text_file(path: &Path) -> bool {
    let mut data: [u8; 2000] = [0; 2000];

    match File::open(path) {
        Ok(mut f) => match read_up_to(&mut f, &mut data) {
            Ok(len) => !data[0..len].contains(&0u8),
            Err(err) => {
                println!(
                    "File {} cannot be read ({}), file ignored",
                    path.display(),
                    err
                );
                false
            }
        },
        Err(err) => {
            println!(
                "File {} cannot be opened ({}), file ignored",
                path.display(),
                err
            );
            false
        }
    }
}

/// Detect type of file
fn detect_file_type(path: &Path) -> FileType {
    if is_text_file(path) {
        return FileType::Text;
    }

    FileType::Unknown
}

/// Check if a file is empty
fn is_file_empty(path: &Path) -> bool {
    match fs::metadata(path).map(|metadata| metadata.len() == 0) {
        Ok(res) => res,
        Err(err) => {
            println!(
                "Emptiness of {} cannot be checked ({}), file ignored",
                path.display(),
                err
            );
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_empty() -> Result<(), String> {
        assert_eq!(is_file_empty(Path::new("./testdata/text_present")), false);
        assert_eq!(is_file_empty(Path::new("./testdata/text_empty_file")), true);
        assert_eq!(is_file_empty(Path::new("./testdata/not_exist")), true);
        Ok(())
    }

    #[test]
    fn test_is_text_file() -> Result<(), String> {
        assert_eq!(is_text_file(Path::new("./testdata/text_present")), true);
        assert_eq!(is_text_file(Path::new("./testdata/text_empty_file")), true);
        assert_eq!(is_text_file(Path::new("./testdata/not_exist")), false);
        assert_eq!(is_text_file(Path::new("./testdata/pdf_empty.pdf")), false);
        Ok(())
    }

    #[test]
    fn test_detect_file_type() -> Result<(), String> {
        assert_eq!(
            detect_file_type(Path::new("./testdata/text_present")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(Path::new("./testdata/text_empty_file")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(Path::new("./testdata/png_empty.png")),
            FileType::Unknown
        );
        Ok(())
    }

    #[test]
    fn test_get_files_list() -> Result<(), String> {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata");
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 3);
        Ok(())
    }

    #[test]
    fn test_get_files_list_filter() -> Result<(), String> {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata");
        config.exclusions = vec![String::from("ignore")];
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 2);
        Ok(())
    }
}
