use indicatif::{ProgressBar, ProgressStyle};
use pdf_oxide::PdfDocument;
use regex::Regex;
use std::{fs, vec};
use walkdir::DirEntry;

use crate::pan_finder::config::*;
use crate::pan_finder::lister::*;

#[derive(Debug)]
pub struct PanFound {
    pub pan: String,
    pub brand: String,
    pub test_bin: bool,
}

#[derive(Debug)]
pub struct FileAnalyseResult {
    pub filename: String,
    pub error_msg: String,
    pub pan_found: Vec<PanFound>,
}

#[derive(Debug)]
pub struct AnalyseResult {
    pub nb_analyzed_file: u64,
    pub nb_error: u64,
    pub nb_found_pan: u64,
    pub results_list: Vec<FileAnalyseResult>,
}
impl AnalyseResult {
    fn new() -> AnalyseResult {
        AnalyseResult {
            nb_analyzed_file: 0,
            nb_error: 0,
            nb_found_pan: 0,
            results_list: Vec::new(),
        }
    }
}

/// Analyse all files
///
/// Return a vector that contains list of files with an analysis error or detected PAN
/// with the error message or list of detected PAN
pub fn analyse_files(files_list: Vec<FilesDescription>, config: &Configuration) -> AnalyseResult {
    let mut analyse_result = AnalyseResult::new();

    let progress_bar = if config.quiet_mode {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(files_list.len() as u64)
    };
    progress_bar.set_prefix("Analyse files");
    progress_bar.set_style(
        ProgressStyle::with_template("{prefix:.bold.dim} {wide_bar} {pos}/{len}").unwrap(),
    );

    let patterns_list = get_patterns();
    for f in files_list {
        progress_bar.inc(1);
        analyse_result.nb_analyzed_file += 1;

        let filename = f.file_entry.path().to_str().unwrap().to_string();
        let result = match f.file_type {
            FileType::Text => analyse_text_file(f.file_entry, &patterns_list, config),
            FileType::Pdf => analyse_pdf_file(f.file_entry, &patterns_list, config),
            FileType::Unknown => {
                Ok(Vec::new())
                // NOP
            }
        };

        match result {
            Ok(pan_found) => {
                if !pan_found.is_empty() {
                    analyse_result.nb_found_pan += pan_found.len() as u64;
                    analyse_result.results_list.push(FileAnalyseResult {
                        filename,
                        error_msg: String::new(),
                        pan_found,
                    })
                }
            }
            Err(error_msg) => {
                analyse_result.nb_error += 1;
                analyse_result.results_list.push(FileAnalyseResult {
                    filename,
                    error_msg,
                    pan_found: Vec::new(),
                })
            }
        }
    }
    progress_bar.finish();

    analyse_result
}

#[derive(Debug)]
struct SubBrand {
    brand: String,
    test_bin: bool,
    bin_list: Vec<String>,
}

#[derive(Debug)]
struct Pattern {
    brand: String,
    re: Regex,
    sub_brand: Vec<SubBrand>,
}

/// Get all supported patterns
fn get_patterns() -> Vec<Pattern> {
    vec![
        Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("CB Test card"),
                    test_bin: true,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("CB Dev card"),
                    test_bin: true,
                    bin_list: vec![String::from("507100")],
                },
                SubBrand {
                    brand: String::from("Visa"),
                    test_bin: false,
                    bin_list: vec![String::from("4")],
                },
                SubBrand {
                    brand: String::from("Mastercard"),
                    test_bin: false,
                    bin_list: vec![
                        String::from("51"),
                        String::from("52"),
                        String::from("53"),
                        String::from("54"),
                        String::from("55"),
                    ],
                },
                SubBrand {
                    brand: String::from("Maestro"),
                    test_bin: false,
                    bin_list: vec![
                        String::from("50"),
                        String::from("56"),
                        String::from("57"),
                        String::from("58"),
                    ],
                },
                SubBrand {
                    brand: String::from("Discover"),
                    test_bin: false,
                    bin_list: vec![
                        String::from("6011"),
                        String::from("64"),
                        String::from("65"),
                        String::from("28"),
                    ],
                },
                SubBrand {
                    brand: String::from("Union pay"),
                    test_bin: false,
                    bin_list: vec![String::from("62")],
                },
                SubBrand {
                    brand: String::from("JCB"),
                    test_bin: false,
                    bin_list: vec![String::from("18"), String::from("35")],
                },
            ],
        },
        Pattern {
            brand: String::from("American Express"),
            re: Regex::new(r"3[-\s]*[47]([-\s]*[0-9]{1}){13}").unwrap(),
            sub_brand: Vec::new(),
        },
        Pattern {
            brand: String::from("Diner's club"),
            re: Regex::new(r"3[-\s]*[0689]([-\s]*[0-9]{1}){12}").unwrap(),
            sub_brand: Vec::new(),
        },
    ]
}

/// Search for a more specific brand
fn search_sub_brand(number: &str, pattern: &Pattern) -> Option<PanFound> {
    for sub_brand in &pattern.sub_brand {
        for bin in &sub_brand.bin_list {
            if number.starts_with(bin) {
                return Some(PanFound {
                    pan: number.to_string(),
                    brand: sub_brand.brand.clone(),
                    test_bin: sub_brand.test_bin,
                });
            }
        }
    }

    None
}

/// Analysis a match result to find the precise card brand if any and detect if its a test BIN range
fn check_match(found_number: &str, pattern: &Pattern, config: &Configuration) -> Option<PanFound> {
    let number: String = found_number
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();

    if luhn::valid(&number) {
        match search_sub_brand(&number, pattern) {
            Some(mut res) => {
                res.pan = found_number.to_string();

                // Remove PAN of test card
                if !config.report_test_bin && res.test_bin {
                    return None;
                } else {
                    return Some(res);
                }
            }
            None => {
                return Some(PanFound {
                    pan: found_number.to_string(),
                    brand: pattern.brand.clone(),
                    test_bin: false,
                });
            }
        }
    }

    None
}

/// Check one of PAN search pattern
fn check_pattern(content: &str, pattern: &Pattern, config: &Configuration) -> Vec<PanFound> {
    let mut results: Vec<PanFound> = Vec::new();

    let matches: Vec<_> = pattern.re.find_iter(content).map(|m| m.as_str()).collect();
    for found_number in matches {
        if let Some(res) = check_match(found_number, pattern, config) {
            results.push(res);
        }
    }

    results
}

/// Search for PAN in a text file
fn analyse_text_file(
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

/// Search for PAN in a text file
fn analyse_pdf_file(
    file: DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<Vec<PanFound>, String> {
    let mut results: Vec<PanFound> = Vec::new();

    let mut doc = match PdfDocument::open(file.path()) {
        Ok(doc) => doc,
        Err(err) => {
            return Err(format!(
                "Can not open PDF file {}: {}",
                file.path().to_str().unwrap(),
                err
            ));
        }
    };

    let nb_page = match doc.page_count() {
        Ok(nb_page) => nb_page,
        Err(err) => {
            return Err(format!(
                "Can not get number of page from {}: {}",
                file.path().to_str().unwrap(),
                err
            ));
        }
    };

    for i in 0..nb_page {
        let content = match doc.extract_text(i) {
            Ok(content) => content,
            Err(err) => {
                return Err(format!(
                    "Can not read page {} from {}: {}",
                    i,
                    file.path().to_str().unwrap(),
                    err
                ));
            }
        };

        for pattern in patterns_list {
            let mut res = check_pattern(&content, pattern, config);
            results.append(&mut res);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use walkdir::WalkDir;

    #[test]
    fn test_search_sub_brand() -> Result<(), String> {
        let pattern = Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: true,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        };

        assert!(search_sub_brand("", &pattern).is_none());
        assert!(search_sub_brand("5117670000000000", &pattern).is_none());
        assert!(!search_sub_brand("5017670000000000", &pattern).is_none());
        let result = search_sub_brand("5017670000000000", &pattern).unwrap();
        assert_eq!(result.brand, String::from("BIN 1"));
        assert_eq!(result.test_bin, true);

        Ok(())
    }

    #[test]
    fn test_check_match_wrong_luhn() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        assert!(check_match("5017670000000001", &pattern, &config).is_none());

        Ok(())
    }

    #[test]
    fn test_check_match_main_entry() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        assert!(!check_match("50671700 00000000", &pattern, &config).is_none());
        let res = check_match("50671700 00000000", &pattern, &config).unwrap();
        assert_eq!(res.brand, "Credit card");
        assert_eq!(res.pan, "50671700 00000000");

        Ok(())
    }

    #[test]
    fn test_check_match_sub_entry() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        assert!(!check_match("50176700 00000000", &pattern, &config).is_none());
        let res = check_match("50176700 00000000", &pattern, &config).unwrap();
        assert_eq!(res.brand, "BIN 1");
        assert_eq!(res.pan, "50176700 00000000");

        Ok(())
    }

    #[test]
    fn test_check_match_test_card_not_reported() -> Result<(), String> {
        let pattern = Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: true,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        };
        let config = Configuration::new();

        assert!(check_match("50176700 00000000", &pattern, &config).is_none());

        Ok(())
    }

    #[test]
    fn test_check_match_test_card_reported() -> Result<(), String> {
        let pattern = Pattern {
            brand: String::from("Credit card"),
            re: Regex::new(r"[2-7]([-\s]*[0-9]{1}){15}").unwrap(),
            sub_brand: vec![
                SubBrand {
                    brand: String::from("BIN 1"),
                    test_bin: true,
                    bin_list: vec![String::from("501767")],
                },
                SubBrand {
                    brand: String::from("BIN 2"),
                    test_bin: false,
                    bin_list: vec![String::from("507100")],
                },
            ],
        };
        let mut config = Configuration::new();
        config.report_test_bin = true;

        assert!(!check_match("50176700 00000000", &pattern, &config).is_none());
        let res = check_match("50176700 00000000", &pattern, &config).unwrap();
        assert_eq!(res.brand, "BIN 1");
        assert_eq!(res.pan, "50176700 00000000");
        assert_eq!(res.test_bin, true);

        Ok(())
    }

    #[test]
    fn test_check_pattern_empty_file() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        let content = "";
        let res = check_pattern(content, &pattern, &config);
        assert!(res.is_empty());

        Ok(())
    }

    #[test]
    fn test_check_pattern_not_present() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        let content = "
                aaa
                bbb
                ccc";
        let res = check_pattern(content, &pattern, &config);
        assert!(res.is_empty());

        Ok(())
    }

    #[test]
    fn test_check_pattern_present() -> Result<(), String> {
        let pattern = Pattern {
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
        };
        let config = Configuration::new();

        let content = "
                aaa
                501767000-0000000
                bbb
                5017670000000001
                ccc";
        let res = check_pattern(content, &pattern, &config);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].pan, "501767000-0000000");

        Ok(())
    }

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

    #[test]
    fn test_analyse_pdf_file_not_present() -> Result<(), String> {
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

        for entry in WalkDir::new("testdata/pdf_not_present.pdf") {
            let res = analyse_pdf_file(entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.is_empty());
        }

        Ok(())
    }

    #[test]
    fn test_analyse_pdf_file_present() -> Result<(), String> {
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

        for entry in WalkDir::new("testdata/pdf_present.pdf") {
            let res = analyse_pdf_file(entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.len(), 2);
            assert_eq!(res[0].pan, "501767000000 0000");
            assert_eq!(res[1].pan, "4017670000000003");
        }

        Ok(())
    }
}
