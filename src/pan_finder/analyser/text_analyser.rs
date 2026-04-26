use std::fs;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::*;
use crate::pan_finder::analyser::common::*;
use crate::pan_finder::config::*;

/// Search for PAN in a text file
pub fn analyse_text_file(
    file: DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<Vec<PanFound>, String> {
    match fs::read_to_string(file.path()) {
        Ok(content) => {
            let mut results: Vec<PanFound> = Vec::new();

            for pattern in patterns_list {
                let mut res = check_pattern(&content, pattern, config);
                results.append(&mut res);
            }
            Ok(results)
        }
        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use walkdir::WalkDir;

    #[test]
    fn test_analyse_text_file_not_present() -> Result<(), String> {
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

        for entry in WalkDir::new("testdata/text_not_present") {
            let res = analyse_text_file(entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.is_empty());
        }

        Ok(())
    }

    #[test]
    fn test_analyse_text_file_present() -> Result<(), String> {
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

        for entry in WalkDir::new("testdata/text_present") {
            let res = analyse_text_file(entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.len(), 2);
            assert_eq!(res[0].pan, "501767000000 0000");
            assert_eq!(res[1].pan, "4017670000000003");
        }

        Ok(())
    }
}
