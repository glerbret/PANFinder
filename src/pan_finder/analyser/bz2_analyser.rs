use bzip2::bufread;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::{FileAnalyseResult, PanFound};
use crate::pan_finder::analyser::common::Pattern;
use crate::pan_finder::analyser::pdf_analyser::analyse_pdf_file_content;
use crate::pan_finder::analyser::tar_analyser::analyse_tar_bz2_file;
use crate::pan_finder::analyser::text_analyser::analyse_text_file_content;
use crate::pan_finder::config::Configuration;
use crate::pan_finder::utils::{is_pdf_file, is_tar_file, is_text_file};

pub fn analyse_bz2_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    match File::open(file.path()) {
        Ok(bz2_file) => {
            let input = BufReader::new(bz2_file);
            let mut decoder = bufread::BzDecoder::new(input);

            let mut buffer = Vec::new();
            let read_size = match decoder.read_to_end(&mut buffer) {
                Ok(size) => size,
                Err(err) => {
                    return Err(format!("Error reading bz2 file: {err}",));
                }
            };

            check_compressed_file(
                patterns_list,
                config,
                file.path().to_str().unwrap(),
                buffer,
                read_size,
                file,
            )
        }
        Err(err) => Err(format!("Error opening bz2 file: {err}")),
    }
}

fn check_compressed_file(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    data: Vec<u8>,
    size: usize,
    file: &DirEntry,
) -> Result<FileAnalyseResult, String> {
    if is_pdf_file(&data, size) {
        if config.check_pdf {
            let res = check_pdf_file(patterns_list, config, filename, data)?;

            Ok(FileAnalyseResult {
                filename: filename.to_string(),
                error_msg: String::new(),
                pan_found: res,
                pan_found_per_subfiles: Vec::new(),
            })
        } else {
            Ok(FileAnalyseResult::new())
        }
    } else if is_tar_file(&data, size) {
        if config.check_tar {
            analyse_tar_bz2_file(file, patterns_list, config)
        } else {
            Ok(FileAnalyseResult::new())
        }
    } else if is_text_file(&data, size) {
        if config.check_text {
            let res = check_text_file(patterns_list, config, filename, &data)?;

            Ok(FileAnalyseResult {
                filename: filename.to_string(),
                error_msg: String::new(),
                pan_found: res,
                pan_found_per_subfiles: Vec::new(),
            })
        } else {
            Ok(FileAnalyseResult::new())
        }
    } else {
        Ok(FileAnalyseResult::new())
    }
}

fn check_text_file(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    data: &[u8],
) -> Result<Vec<PanFound>, String> {
    match str::from_utf8(data) {
        Ok(data) => Ok(analyse_text_file_content(
            patterns_list,
            config,
            filename,
            data,
        )),
        Err(err) => Err(format!("Invalid UTF-8 sequence in {filename}, {err}")),
    }
}

fn check_pdf_file(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    data: Vec<u8>,
) -> Result<Vec<PanFound>, String> {
    analyse_pdf_file_content(patterns_list, config, filename, data)
}

#[cfg(test)]
mod tests {
    use crate::pan_finder::analyser::common::SubBrand;

    use super::*;
    use regex::Regex;
    use walkdir::WalkDir;

    #[test]
    fn analyse_bz2_file_txt_not_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/text_not_present.bz2") {
            let res: FileAnalyseResult =
                analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn analyse_bz2_file_pdf_not_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/pdf_not_present.bz2") {
            let res = analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn analyse_bz2_file_txt_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/text_present.bz2") {
            let res: FileAnalyseResult =
                analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.pan_found.len(), 4);
            assert_eq!(res.pan_found[0].pan, "************0000");
            assert_eq!(res.pan_found[1].pan, "************0018");
            assert_eq!(res.pan_found[2].pan, "************0026");
            assert_eq!(res.pan_found[3].pan, "************0034");
        }
    }

    #[test]
    fn analyse_bz2_file_pdf_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/pdf_present.bz2") {
            let res = analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.pan_found.len(), 5);
            assert_eq!(res.pan_found[0].pan, "************0000");
            assert_eq!(res.pan_found[1].pan, "************0018");
            assert_eq!(res.pan_found[2].pan, "************0026");
            assert_eq!(res.pan_found[3].pan, "************0034");
            assert_eq!(res.pan_found[4].pan, "************0042");
        }
    }

    #[test]
    fn analyse_bz2_file_tar_not_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/tar_not_present.tar.bz2") {
            let res = analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn analyse_bz2_file_tar_present() {
        let patterns = vec![Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: false,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        }];
        let config = Configuration::new();

        for entry in WalkDir::new("testdata/tar_present.tar.bz2") {
            let res = analyse_bz2_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.pan_found.len(), 0);
            assert_eq!(res.pan_found_per_subfiles.len(), 2);

            assert_eq!(res.pan_found_per_subfiles[0].subfilename, "pdf_present.pdf");
            assert_eq!(res.pan_found_per_subfiles[0].pan_found.len(), 5);
            assert_eq!(
                res.pan_found_per_subfiles[0].pan_found[0].pan,
                "************0000"
            );
            assert_eq!(
                res.pan_found_per_subfiles[0].pan_found[1].pan,
                "************0018"
            );
            assert_eq!(
                res.pan_found_per_subfiles[0].pan_found[2].pan,
                "************0026"
            );
            assert_eq!(
                res.pan_found_per_subfiles[0].pan_found[3].pan,
                "************0034"
            );
            assert_eq!(
                res.pan_found_per_subfiles[0].pan_found[4].pan,
                "************0042"
            );

            assert_eq!(
                res.pan_found_per_subfiles[1].subfilename,
                "text_present.txt"
            );
            assert_eq!(res.pan_found_per_subfiles[1].pan_found.len(), 4);
            assert_eq!(
                res.pan_found_per_subfiles[1].pan_found[0].pan,
                "************0000"
            );
            assert_eq!(
                res.pan_found_per_subfiles[1].pan_found[1].pan,
                "************0018"
            );
            assert_eq!(
                res.pan_found_per_subfiles[1].pan_found[2].pan,
                "************0026"
            );
            assert_eq!(
                res.pan_found_per_subfiles[1].pan_found[3].pan,
                "************0034"
            );
        }
    }
}
