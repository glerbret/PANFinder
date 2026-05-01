use std::fs;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::{FileAnalyseResult, PanFound};
use crate::pan_finder::analyser::common::{Pattern, check_pattern};
use crate::pan_finder::config::Configuration;

/// Search for PAN in a text file
pub fn analyse_text_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    match fs::read_to_string(file.path()) {
        Ok(content) => Ok(FileAnalyseResult {
            filename: file.path().to_str().unwrap().to_string(),
            error_msg: String::new(),
            pan_found: analyse_text_file_content(
                patterns_list,
                config,
                file.path().to_str().unwrap(),
                &content,
            ),
            pan_found_per_subfiles: Vec::new(),
        }),
        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}

/// Analyse text file content
pub fn analyse_text_file_content(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    content: &str,
) -> Vec<PanFound> {
    let mut results: Vec<PanFound> = Vec::new();

    for pattern in patterns_list {
        let mut res = check_pattern(content, pattern, config, filename);
        results.append(&mut res);
    }

    results
}

#[cfg(test)]
mod tests {
    use crate::pan_finder::analyser::common::SubBrand;

    use super::*;
    use regex::Regex;
    use walkdir::WalkDir;

    #[test]
    fn test_analyse_text_file_not_present() {
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

        for entry in WalkDir::new("testdata/text_not_present.txt") {
            let res = analyse_text_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.pan_found.is_empty());
        }
    }

    #[test]
    fn test_analyse_text_file_present() {
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

        for entry in WalkDir::new("testdata/text_present.txt") {
            let res = analyse_text_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.pan_found.len(), 4);
            assert_eq!(res.pan_found[0].pan, "5017670000000000");
            assert_eq!(res.pan_found[1].pan, "5017670000000018");
            assert_eq!(res.pan_found[2].pan, "5017670000000026");
            assert_eq!(res.pan_found[3].pan, "5017670000000034");
        }
    }
}
