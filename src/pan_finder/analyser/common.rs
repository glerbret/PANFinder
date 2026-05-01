use regex::Regex;
use std::collections::HashMap;

use crate::pan_finder::analyser::analyser_api::PanFound;
use crate::pan_finder::config::Configuration;

#[derive(Debug)]
pub struct SubBrand {
    pub brand: String,
    pub test_bin: bool,
    pub bin_list: Vec<String>,
}

#[derive(Debug)]
pub struct Pattern {
    pub brand: String,
    pub re: Regex,
    pub sub_brand: Vec<SubBrand>,
}

/// Search for a more specific brand
pub fn search_sub_brand(
    number: &str,
    pattern: &Pattern,
    config: &Configuration,
) -> Option<PanFound> {
    for sub_brand in &pattern.sub_brand {
        for bin in &sub_brand.bin_list {
            if number.starts_with(bin) {
                return Some(PanFound {
                    pan: truncate_card_number(config, number),
                    brand: sub_brand.brand.clone(),
                    test_bin: sub_brand.test_bin,
                });
            }
        }
    }

    None
}

/// Analysis a match result to find the precise card brand if any and detect if its a test BIN range
pub fn check_match(
    found_number: &str,
    pattern: &Pattern,
    config: &Configuration,
    filename: &str,
) -> Option<PanFound> {
    let number: String = found_number
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();

    if luhn::valid(&number)
        && !is_excluded(
            &number,
            filename,
            &config.excluded_pan,
            &config.excluded_pan_per_file,
        )
    {
        match search_sub_brand(&number, pattern, config) {
            Some(res) => {
                // Remove PAN of test card
                if !config.report_test_bin && res.test_bin {
                    return None;
                }

                return Some(res);
            }
            None => {
                return Some(PanFound {
                    pan: truncate_card_number(config, &number),
                    brand: pattern.brand.clone(),
                    test_bin: false,
                });
            }
        }
    }

    None
}

/// Truncate card number in report (only last 4 digits are used)
pub fn truncate_card_number(config: &Configuration, number: &str) -> String {
    if config.truncated_pan {
        let mut truncated = "*******************".to_string();
        truncated.truncate(number.len() - 4);
        truncated.push_str(&number[number.len() - 4..]);
        truncated
    } else {
        number.to_string()
    }
}

/// Check one of PAN search pattern
pub fn check_pattern(
    content: &str,
    pattern: &Pattern,
    config: &Configuration,
    filename: &str,
) -> Vec<PanFound> {
    let mut results: Vec<PanFound> = Vec::new();

    let matches: Vec<_> = pattern.re.find_iter(content).map(|m| m.as_str()).collect();
    for found_number in matches {
        if let Some(res) = check_match(found_number, pattern, config, filename) {
            results.push(res);
        }
    }

    results
}

/// Exclude some PAN from result search (global and per file)
fn is_excluded(
    number: &str,
    filename: &str,
    excluded_pan: &Vec<String>,
    excluded_pan_per_file: &HashMap<String, Vec<String>>,
) -> bool {
    for pan in excluded_pan {
        if number.starts_with(pan) {
            return true;
        }
    }

    if !excluded_pan_per_file.is_empty() {
        // Replace \ per / to accept both Linux and Windows path format
        let key: String = filename
            .chars()
            .map(|c| if c == '\\' { '/' } else { c })
            .collect();

        if excluded_pan_per_file.contains_key(&key) {
            for pan in &excluded_pan_per_file[&key] {
                if number.starts_with(pan) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_sub_brand() {
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

        assert!(search_sub_brand("", &pattern, &Configuration::new()).is_none());
        assert!(search_sub_brand("5117670000000000", &pattern, &Configuration::new()).is_none());
        assert!(search_sub_brand("5017670000000000", &pattern, &Configuration::new()).is_some());
        let result = search_sub_brand("5017670000000000", &pattern, &Configuration::new()).unwrap();
        assert_eq!(result.brand, String::from("BIN 1"));
        assert!(result.test_bin);
    }

    #[test]
    fn test_check_match_wrong_luhn() {
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

        assert!(check_match("5017670000000001", &pattern, &config, "").is_none());
    }

    #[test]
    fn test_check_match_excluded_pan_global() {
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

        {
            let mut config = Configuration::new();
            config.excluded_pan = vec![String::from("5017670000000000")];
            assert!(check_match("5017670000000000", &pattern, &config, "").is_none());
        }

        {
            let mut config = Configuration::new();
            config.excluded_pan = vec![String::from("5017670000000000")];
            assert!(check_match("5017670000000000", &pattern, &config, "").is_none());
        }

        {
            let mut config = Configuration::new();
            config.excluded_pan = vec![String::from("50176700")];
            assert!(check_match("5017670000000000", &pattern, &config, "").is_none());
        }

        {
            let mut config = Configuration::new();
            config.excluded_pan = vec![String::from("5017670000000018")];
            assert!(check_match("5017670000000000", &pattern, &config, "").is_some());
        }
    }

    #[test]
    fn test_check_match_excluded_pan_per_file() {
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

        {
            let mut excluded_pan_per_file = HashMap::new();
            excluded_pan_per_file
                .insert(String::from("file"), vec![String::from("5017670000000000")]);
            let mut config = Configuration::new();
            config.excluded_pan_per_file = excluded_pan_per_file;
            assert!(check_match("5017670000000000", &pattern, &config, "file").is_none());
        }

        {
            let mut excluded_pan_per_file = HashMap::new();
            excluded_pan_per_file
                .insert(String::from("file"), vec![String::from("5017670000000000")]);
            let mut config = Configuration::new();
            config.excluded_pan_per_file = excluded_pan_per_file;
            assert!(check_match("5017670000000000", &pattern, &config, "file").is_none());
        }

        {
            let mut excluded_pan_per_file = HashMap::new();
            excluded_pan_per_file.insert(String::from("file"), vec![String::from("501767")]);
            let mut config = Configuration::new();
            config.excluded_pan_per_file = excluded_pan_per_file;
            assert!(check_match("5017670000000000", &pattern, &config, "file").is_none());
        }

        {
            let mut excluded_pan_per_file = HashMap::new();
            excluded_pan_per_file
                .insert(String::from("file"), vec![String::from("5017670000000018")]);
            let mut config = Configuration::new();
            config.excluded_pan_per_file = excluded_pan_per_file;
            assert!(check_match("5017670000000000", &pattern, &config, "file").is_some());
        }

        {
            let mut excluded_pan_per_file = HashMap::new();
            excluded_pan_per_file
                .insert(String::from("file"), vec![String::from("5017670000000000")]);
            let mut config = Configuration::new();
            config.excluded_pan_per_file = excluded_pan_per_file;
            assert!(check_match("5017670000000000", &pattern, &config, "other").is_some());
        }
    }

    #[test]
    fn test_check_match_main_entry() {
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

        assert!(check_match("50671700 00000000", &pattern, &config, "").is_some());
        let res = check_match("50671700 00000000", &pattern, &config, "").unwrap();
        assert_eq!(res.brand, "Credit card");
        assert_eq!(res.pan, "************0000");
    }

    #[test]
    fn test_check_match_sub_entry() {
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

        assert!(check_match("50176700 00000000", &pattern, &config, "").is_some());
        let res = check_match("50176700 00000000", &pattern, &config, "").unwrap();
        assert_eq!(res.brand, "BIN 1");
        assert_eq!(res.pan, "************0000");
    }

    #[test]
    fn test_check_match_test_card_not_reported() {
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

        assert!(check_match("50176700 00000000", &pattern, &config, "").is_none());
    }

    #[test]
    fn test_check_match_test_card_reported() {
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

        assert!(check_match("50176700 00000000", &pattern, &config, "").is_some());
        let res = check_match("50176700 00000000", &pattern, &config, "").unwrap();
        assert_eq!(res.brand, "BIN 1");
        assert_eq!(res.pan, "************0000");
        assert!(res.test_bin);
    }

    #[test]
    fn test_check_pattern_empty_file() {
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
        let res = check_pattern(content, &pattern, &config, "");
        assert!(res.is_empty());
    }

    #[test]
    fn test_check_pattern_not_present() {
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
        let res = check_pattern(content, &pattern, &config, "");
        assert!(res.is_empty());
    }

    #[test]
    fn test_check_pattern_present() {
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
        let res = check_pattern(content, &pattern, &config, "");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].pan, "************0000");
    }

    #[test]
    fn test_check_pattern_present_not_mask() {
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
        let mut config = Configuration::new();
        config.truncated_pan = false;

        let content = "
                aaa
                501767000-0000000
                bbb
                5017670000000001
                ccc";
        let res = check_pattern(content, &pattern, &config, "");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].pan, "5017670000000000");
    }
}
