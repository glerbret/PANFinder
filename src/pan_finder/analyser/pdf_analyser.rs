use pdf_oxide::PdfDocument;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::PanFound;
use crate::pan_finder::analyser::common::{Pattern, check_pattern};
use crate::pan_finder::config::Configuration;

/// Search for PAN in a text file
pub fn analyse_pdf_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<Vec<PanFound>, String> {
    let mut results: Vec<PanFound> = Vec::new();

    let doc = get_pdf_doc(file)?;
    let nb_pages = get_pdf_number_pages(file, &doc)?;

    for i in 0..nb_pages {
        match doc.extract_text(i) {
            Ok(content) => {
                for pattern in patterns_list {
                    let mut res =
                        check_pattern(&content, pattern, config, file.path().to_str().unwrap());
                    results.append(&mut res);
                }
            }
            Err(err) => {
                return Err(format!(
                    "Can not read page {} from {}: {}",
                    i,
                    file.path().to_str().unwrap(),
                    err
                ));
            }
        }
    }

    Ok(results)
}

fn get_pdf_doc(file: &DirEntry) -> Result<PdfDocument, String> {
    match PdfDocument::open(file.path()) {
        Ok(doc) => Ok(doc),
        Err(err) => Err(format!(
            "Can not open PDF file {}: {}",
            file.path().to_str().unwrap(),
            err
        )),
    }
}

fn get_pdf_number_pages(file: &DirEntry, doc: &PdfDocument) -> Result<usize, String> {
    match doc.page_count() {
        Ok(nb_pages) => Ok(nb_pages),
        Err(err) => Err(format!(
            "Can not get number of page from {}: {}",
            file.path().to_str().unwrap(),
            err
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::pan_finder::analyser::common::SubBrand;

    use super::*;
    use regex::Regex;
    use walkdir::WalkDir;

    #[test]
    fn test_analyse_pdf_file_not_present() {
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
            let res = analyse_pdf_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert!(res.is_empty());
        }
    }

    #[test]
    fn test_analyse_pdf_file_present() {
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
            let res = analyse_pdf_file(&entry.unwrap(), &patterns, &config).unwrap();
            assert_eq!(res.len(), 5);
            assert_eq!(res[0].pan, "5017670000000000");
            assert_eq!(res[1].pan, "5017670 000000018");
            assert_eq!(res[2].pan, "50176700000000-26");
            assert_eq!(res[3].pan, "50176 \n70000000034");
            assert_eq!(res[4].pan, "5017670000000042");
        }
    }
}
