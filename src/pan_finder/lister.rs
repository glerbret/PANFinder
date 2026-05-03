use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::path::Path;
use walkdir::DirEntry;
use walkdir::WalkDir;

use crate::pan_finder::config::Configuration;
use crate::pan_finder::utils::{
    is_bz2_file, is_file_empty, is_gz_file, is_pdf_file, is_tar_file, is_text_file, is_zip_file,
    read_up_to,
};

#[derive(PartialEq, Eq, Debug)]
pub enum FileType {
    Unknown,

    Text,
    Pdf,
    Tar,
    Gzip,
    Bzip2,
    Zip,
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
        .filter_entry(|entry| !is_excluded(entry, &config.excluded_path))
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
            } else if is_tar_file(&data, len) {
                if config.check_tar {
                    FileType::Tar
                } else {
                    FileType::Unknown
                }
            } else if is_gz_file(&data, len) {
                if config.check_compress {
                    FileType::Gzip
                } else {
                    FileType::Unknown
                }
            } else if is_bz2_file(&data, len) {
                if config.check_compress {
                    FileType::Bzip2
                } else {
                    FileType::Unknown
                }
            } else if is_zip_file(&data, len) {
                if config.check_compress {
                    FileType::Zip
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_file_empty() {
        assert!(!is_file_empty(Path::new("./testdata/lister/text_file.txt")));
        assert!(is_file_empty(Path::new("./testdata/lister/empty_file")));
        assert!(is_file_empty(Path::new("./testdata/lister/not_exist")));
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
    fn test_is_tar_file() {
        let mut data: [u8; 2000] = [0x30; 2000];
        assert!(!is_tar_file(&data, 500));

        data[257] = b'u';
        data[258] = b's';
        data[259] = b't';
        data[260] = b'a';
        data[261] = b'r';
        assert!(is_tar_file(&data, 500));
        assert!(!is_tar_file(&data, 140));
    }

    #[test]
    fn test_detect_file_type_all() {
        let config = Configuration::new();

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/text_file.txt")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/png_file.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/pdf_file.pdf")),
            FileType::Pdf
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/tar_file.tar")),
            FileType::Tar
        );
    }

    #[test]
    fn test_detect_file_type_no_text() {
        let mut config = Configuration::new();
        config.check_text = false;

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/text_file.txt")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/png_file.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/pdf_file.pdf")),
            FileType::Pdf
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/tar_file.tar")),
            FileType::Tar
        );
    }

    #[test]
    fn test_detect_file_type_no_pdf() {
        let mut config = Configuration::new();
        config.check_pdf = false;

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/text_file.txt")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/png_file.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/pdf_file.pdf")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/tar_file.tar")),
            FileType::Tar
        );
    }

    #[test]
    fn test_detect_file_type_no_tar() {
        let mut config = Configuration::new();
        config.check_tar = false;

        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/text_file.txt")),
            FileType::Text
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/png_file.png")),
            FileType::Unknown
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/pdf_file.pdf")),
            FileType::Pdf
        );
        assert_eq!(
            detect_file_type(&config, Path::new("./testdata/lister/tar_file.tar")),
            FileType::Unknown
        );
    }

    #[test]
    fn test_get_files_list() {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata/lister");
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 4);
    }

    #[test]
    fn test_get_files_list_filter() {
        let mut config = Configuration::new();
        config.search_dir = String::from("testdata/lister");
        config.excluded_path = vec![String::from("ignore")];
        config.quiet_mode = true;

        let res = get_files_list(&config);

        assert_eq!(res.len(), 3);
    }
}
