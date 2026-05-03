use std::fs::File;
use std::io::Read;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::{
    FileAnalyseResult, PanFound, SubFileAnalyseResult,
};
use crate::pan_finder::analyser::common::Pattern;
use crate::pan_finder::analyser::pdf_analyser::analyse_pdf_file_content;
use crate::pan_finder::analyser::text_analyser::analyse_text_file_content;
use crate::pan_finder::config::Configuration;
use crate::pan_finder::utils::{is_pdf_file, is_text_file};

pub fn analyse_zip_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    let mut results = FileAnalyseResult {
        filename: file.path().to_str().unwrap().to_string(),
        error_msg: String::new(),
        pan_found: Vec::new(),
        pan_found_per_subfiles: Vec::new(),
    };

    match File::open(file.path()) {
        Ok(zip_file) => {
            let mut zip = match zip::ZipArchive::new(zip_file) {
                Ok(zip) => zip,
                Err(err) => return Err(format!("Error opening zip file: {err}")),
            };

            for i in 0..zip.len() {
                let mut inc_file = match zip.by_index(i) {
                    Ok(inc_file) => inc_file,
                    Err(err) => return Err(format!("Error getting included file: {err}")),
                };

                if inc_file.is_file() {
                    let mut data = Vec::new();
                    let size = match inc_file.read_to_end(&mut data) {
                        Ok(size) => size,
                        Err(err) => {
                            return Err(format!(
                                "Error reading included file {}: {}",
                                inc_file.name(),
                                err
                            ));
                        }
                    };

                    match check_inc_file(patterns_list, config, inc_file.name(), data, size) {
                        Ok(pan_found) => {
                            if !pan_found.is_empty() {
                                results.pan_found_per_subfiles.push(SubFileAnalyseResult {
                                    subfilename: inc_file.name().to_string(),
                                    pan_found,
                                });
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }
            }
        }
        Err(err) => return Err(format!("Error opening file {}: {}", results.filename, err)),
    }

    Ok(results)
}

fn check_inc_file(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    data: Vec<u8>,
    size: usize,
) -> Result<Vec<PanFound>, String> {
    if is_pdf_file(&data, size) {
        if config.check_pdf {
            check_pdf_file(patterns_list, config, filename, data)
        } else {
            Ok(Vec::new())
        }
    } else if is_text_file(&data, size) {
        if config.check_text {
            check_text_file(patterns_list, config, filename, &data)
        } else {
            Ok(Vec::new())
        }
    } else {
        Ok(Vec::new())
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
    fn test_analyse_zip_file_not_present() {
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

        for entry in WalkDir::new("testdata/zip_not_present.zip") {
            let res = analyse_zip_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn test_analyse_zip_file_present() {
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

        for entry in WalkDir::new("testdata/zip_present.zip") {
            let res = analyse_zip_file(&entry.unwrap(), &patterns, &config).unwrap();
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

    #[test]
    fn test_analyse_zip_with_dir() {
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

        for entry in WalkDir::new("testdata/zip_with_dir.zip") {
            let res = analyse_zip_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.pan_found.len(), 0);
            assert_eq!(res.pan_found_per_subfiles.len(), 2);

            assert_eq!(
                res.pan_found_per_subfiles[0].subfilename,
                "zip_with_dir/pdf/pdf_present.pdf"
            );
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
                "zip_with_dir/txt/text_present.txt"
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
