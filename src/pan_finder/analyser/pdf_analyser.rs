use pdf_oxide::PdfDocument;
use walkdir::DirEntry;

use crate::pan_finder::analyser::analyser_api::{FileAnalyseResult, PanFound};
use crate::pan_finder::analyser::common::{Pattern, check_pattern};
use crate::pan_finder::config::Configuration;

/// Search for PAN in a PDF file
pub fn analyse_pdf_file(
    file: &DirEntry,
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
) -> Result<FileAnalyseResult, String> {
    let doc = get_pdf_doc_from_file(file)?;
    match analyse_content(patterns_list, config, file.path().to_str().unwrap(), &doc) {
        Ok(pan) => Ok(FileAnalyseResult {
            filename: file.path().to_str().unwrap().to_string(),
            error_msg: String::new(),
            pan_found: pan,
            pan_found_per_subfiles: Vec::new(),
        }),
        Err(e) => Err(e),
    }
}

/// Search for PAN in a PDF file (provided as byte array)
pub fn analyse_pdf_file_content(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    content: Vec<u8>,
) -> Result<Vec<PanFound>, String> {
    let doc = get_pdf_doc_from_bytes(filename, content)?;
    analyse_content(patterns_list, config, filename, &doc)
}

fn get_pdf_doc_from_file(file: &DirEntry) -> Result<PdfDocument, String> {
    match PdfDocument::open(file.path()) {
        Ok(doc) => Ok(doc),
        Err(err) => Err(format!(
            "Can not open PDF file {}: {}",
            file.path().to_str().unwrap(),
            err
        )),
    }
}

fn get_pdf_doc_from_bytes(filename: &str, data: Vec<u8>) -> Result<PdfDocument, String> {
    match PdfDocument::from_bytes(data) {
        Ok(doc) => Ok(doc),
        Err(err) => Err(format!("Can not open PDF file {filename}: {err}")),
    }
}

fn analyse_content(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    filename: &str,
    doc: &PdfDocument,
) -> Result<Vec<PanFound>, String> {
    let mut results: Vec<PanFound> = Vec::new();
    let nb_pages = get_pdf_number_pages(filename, doc)?;

    for i in 0..nb_pages {
        match doc.extract_text(i) {
            Ok(content) => {
                for pattern in patterns_list {
                    let mut res = check_pattern(&content, pattern, config, filename);
                    results.append(&mut res);
                }
            }
            Err(err) => {
                return Err(format!("Can not read page {i} from {filename}: {err}"));
            }
        }
    }

    Ok(results)
}

fn get_pdf_number_pages(filename: &str, doc: &PdfDocument) -> Result<usize, String> {
    match doc.page_count() {
        Ok(nb_pages) => Ok(nb_pages),
        Err(err) => Err(format!("Can not get number of page from {filename}: {err}")),
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
            assert!(res.pan_found.is_empty());
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
            assert_eq!(res.pan_found.len(), 5);
            assert_eq!(res.pan_found[0].pan, "************0000");
            assert_eq!(res.pan_found[1].pan, "************0018");
            assert_eq!(res.pan_found[2].pan, "************0026");
            assert_eq!(res.pan_found[3].pan, "************0034");
            assert_eq!(res.pan_found[4].pan, "************0042");
        }
    }
}
