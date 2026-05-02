use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs::File;
use tar::{Archive, Entry};
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::{
    FileAnalyseResult, PanFound, SubFileAnalyseResult,
};
use crate::pan_finder::analyser::common::Pattern;
use crate::pan_finder::analyser::pdf_analyser::analyse_pdf_file_content;
use crate::pan_finder::analyser::text_analyser::analyse_text_file_content;
use crate::pan_finder::config::Configuration;
use crate::pan_finder::utils::{is_pdf_file, is_text_file, read_up_to};

pub fn analyse_tar_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    match File::open(file.path()) {
        Ok(tar_file) => {
            let mut archive = Archive::new(tar_file);
            check_tar_file(
                patterns_list,
                config,
                file.path().to_str().unwrap(),
                &mut archive,
            )
        }

        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}

pub fn analyse_tar_gz_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    match File::open(file.path()) {
        Ok(tar_file) => {
            let tar = GzDecoder::new(tar_file);
            let mut archive = Archive::new(tar);

            check_tar_file(
                patterns_list,
                config,
                file.path().to_str().unwrap(),
                &mut archive,
            )
        }

        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}

pub fn analyse_tar_bz2_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    match File::open(file.path()) {
        Ok(tar_file) => {
            let tar = BzDecoder::new(tar_file);
            let mut archive = Archive::new(tar);

            check_tar_file(
                patterns_list,
                config,
                file.path().to_str().unwrap(),
                &mut archive,
            )
        }

        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}

fn check_tar_file<T: std::io::Read>(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    archive: &mut Archive<T>,
) -> Result<FileAnalyseResult, String> {
    let mut results = FileAnalyseResult {
        filename: filename.to_string(),
        error_msg: String::new(),
        pan_found: Vec::new(),
        pan_found_per_subfiles: Vec::new(),
    };

    //let mut archive = Archive::new(tar_file);

    let archive_content = match archive.entries() {
        Ok(archive_content) => archive_content,
        Err(e) => {
            return Err(format!("read entries error {filename} {e}"));
        }
    };

    for inc_file in archive_content {
        let mut inc_file = match inc_file {
            Ok(in_file) => in_file,
            Err(e) => {
                return Err(format!("read embedded file error {filename} {e}"));
            }
        };

        match check_inc_file(patterns_list, config, &mut inc_file) {
            Ok(pan_found) => {
                if !pan_found.is_empty() {
                    results.pan_found_per_subfiles.push(SubFileAnalyseResult {
                        subfilename: inc_file
                            .header()
                            .path()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string(),
                        pan_found,
                    });
                }
            }
            Err(e) => return Err(e),
        }

        //results.append(&mut check_inc_file(patterns_list, config, &mut inc_file)?);
    }

    Ok(results)
}

fn check_inc_file<T: std::io::Read>(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    inc_file: &mut Entry<'_, T>,
) -> Result<Vec<PanFound>, String> {
    let size = usize::try_from(inc_file.header().size().unwrap()).unwrap();
    let mut data = vec![0u8; size];

    match read_up_to(inc_file, &mut data) {
        Ok(_) => {
            let filename = inc_file.header().path().unwrap();
            if is_pdf_file(&data, size) {
                check_pdf_file(patterns_list, config, filename.to_str().unwrap(), data)
            } else if is_text_file(&data, size) {
                check_text_file(patterns_list, config, filename.to_str().unwrap(), &data)
            } else {
                Ok(Vec::new())
            }
        }
        Err(e) => Err(format!(
            "read error {} {}",
            inc_file.header().path().unwrap().to_str().unwrap(),
            e
        )),
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
        Err(e) => Err(format!("Invalid UTF-8 sequence in {filename}, {e}")),
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
    fn test_analyse_tar_file_not_present() {
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

        for entry in WalkDir::new("testdata/tar_not_present.tar") {
            let res = analyse_tar_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn test_analyse_tar_file_present() {
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

        for entry in WalkDir::new("testdata/tar_present.tar") {
            let res = analyse_tar_file(&entry.unwrap(), &patterns, &config).unwrap();
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
