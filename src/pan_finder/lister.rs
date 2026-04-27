use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::fs::File;
use std::path::Path;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::pan_finder::config::Configuration;

#[derive(PartialEq, Eq, Debug)]
pub enum FileType {
    Unknown,

    Text,
    Pdf,
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
    let progress_bar = init_progress_bar(config.quiet_mode);

    for entry in WalkDir::new(&config.search_dir)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry, &config.exclusions))
    {
        progress_bar.set_message("In progress...");
        progress_bar.inc(1);
        let ent = entry.unwrap();
        if ent.file_type().is_file() && !is_file_empty(ent.path()) {
            let file_type = detect_file_type(config, ent.path());

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

fn init_progress_bar(quiet_mode: bool) -> ProgressBar {
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    let progress_bar = if quiet_mode {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(10)
    };
    progress_bar.set_style(spinner_style);
    progress_bar.set_prefix("Build list of files");

    progress_bar
}

/// Check is a file or a directory is excluded from analyse
fn is_excluded(entry: &DirEntry, exclusions: &Vec<String>) -> bool {
    entry.path().to_str().is_some_and(|s| {
        for exclusion in exclusions {
            if s.contains(exclusion) {
                return true;
            }
        }
        false
    })
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

/// Read file content
fn read_file_content(path: &Path, data: &mut [u8; 2000]) -> Result<usize, String> {
    match File::open(path) {
        Ok(mut f) => match read_up_to(&mut f, data) {
            Ok(len) => Ok(len),
            Err(err) => Err(format!(
                "File {} cannot be read ({}), file ignored",
                path.display(),
                err
            )),
        },
        Err(err) => Err(format!(
            "File {} cannot be opened ({}), file ignored",
            path.display(),
            err
        )),
    }
}

/// Check is a file is a text one
///
/// _Note:_ a file is considered be a text one if there is no `0` in its first 8000 bytes
fn is_text_file(data: &[u8; 2000], len: usize) -> bool {
    !data[0..len].contains(&0u8)
}

/// Check is a file is a text one
fn is_pdf_file(data: &[u8; 2000], len: usize) -> bool {
    if len >= 4 {
        let header: [u8; 4] = data[0..4].try_into().unwrap();
        &header == b"%PDF"
    } else {
        false
    }
}

/// Detect type of file
fn detect_file_type(config: &Configuration, path: &Path) -> FileType {
    let mut data: [u8; 2000] = [0; 2000];
    match read_file_content(path, &mut data) {
        Ok(len) => {
            if is_pdf_file(&data, len) {
                if config.check_pdf {
                    FileType::Pdf
                } else {
                    FileType::Unknown
                }
            } else if config.check_text && is_text_file(&data, len) {
                FileType::Text
            } else {
                FileType::Unknown
            }
        }
        Err(error) => {
            println!("{error}");
            FileType::Unknown
        }
    }
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
    fn test_is_file_empty() {
        assert!(!is_file_empty(Path::new("./testdata/text_present")));
        assert!(is_file_empty(Path::new("./testdata/text_empty_file")));
        assert!(is_file_empty(Path::new("./testdata/not_exist")));
    }

    #[test]
    fn test_is_text_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        data[150] = 0;

        assert!(is_text_file(&data, 100));
        assert!(is_text_file(&data, 0));
        assert!(!is_text_file(&data, 200));
    }

    #[test]
    fn test_is_pdf_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        assert!(!is_pdf_file(&data, 100));

        data[0] = b'%';
        data[1] = b'P';
        data[2] = b'D';
        data[3] = b'F';
        assert!(is_pdf_file(&data, 100));
        assert!(!is_pdf_file(&data, 3));
    }

    #[test]
    fn test_detect_file_type_all() {
        let config = Configuration::new();

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_present")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_empty_file")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/png_empty.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_empty.pdf")),
            FileType::Pdf
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_present.pdf")),
            FileType::Pdf
        );
    }

    #[test]
    fn test_detect_file_type_no_text() {
        let mut config = Configuration::new();
        config.check_text = false;

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_present")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_empty_file")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/png_empty.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_empty.pdf")),
            FileType::Pdf
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_present.pdf")),
            FileType::Pdf
        );
    }

    #[test]
    fn test_detect_file_type_no_pdf() {
        let mut config = Configuration::new();
        config.check_pdf = false;

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_present")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/text_empty_file")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/png_empty.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_empty.pdf")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/pdf_present.pdf")),
            FileType::Unknown
        );
    }

    #[test]
    fn test_get_files_list() {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata");
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 8);
    }

    #[test]
    fn test_get_files_list_filter() {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata");
        config.exclusions = vec![String::from("ignore")];
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 7);
    }
}
