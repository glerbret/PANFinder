use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::vec;

use crate::pan_finder::analyser::common::{Pattern, SubBrand};
use crate::pan_finder::analyser::gz_analyser::analyse_gz_file;
use crate::pan_finder::analyser::pdf_analyser::analyse_pdf_file;
use crate::pan_finder::analyser::tar_analyser::analyse_tar_file;
use crate::pan_finder::analyser::text_analyser::analyse_text_file;
use crate::pan_finder::config::Configuration;
use crate::pan_finder::lister::{FileType, FilesDescription};

#[derive(Debug)]
pub struct PanFound {
    pub pan: String,
    pub brand: String,
    pub test_bin: bool,
}

#[derive(Debug)]
pub struct SubFileAnalyseResult {
    pub subfilename: String,
    pub pan_found: Vec<PanFound>,
}

#[derive(Debug)]
pub struct FileAnalyseResult {
    pub filename: String,
    pub error_msg: String,
    pub pan_found: Vec<PanFound>,
    // If file contains subfile
    pub pan_found_per_subfiles: Vec<SubFileAnalyseResult>,
}

#[derive(Debug)]
pub struct AnalyseResult {
    pub nb_analyzed_file: u64,
    pub nb_error: u64,
    pub nb_found_pan: u64,
    pub results_list: Vec<FileAnalyseResult>,
}
impl AnalyseResult {
    const fn new() -> Self {
        Self {
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
    let progress_bar = init_progress_bar(config.quiet_mode, files_list.len() as u64);

    let patterns_list = get_patterns();
    for f in files_list {
        progress_bar.inc(1);
        analyse_result.nb_analyzed_file += 1;

        let filename = f.file_entry.path().to_str().unwrap().to_string();
        let result = match f.file_type {
            FileType::Text => analyse_text_file(&f.file_entry, &patterns_list, config),
            FileType::Pdf => analyse_pdf_file(&f.file_entry, &patterns_list, config),
            FileType::Tar => analyse_tar_file(&f.file_entry, &patterns_list, config),
            FileType::Gzip => analyse_gz_file(&f.file_entry, &patterns_list, config),
            FileType::Unknown => Ok(FileAnalyseResult {
                filename: String::new(),
                error_msg: String::new(),
                pan_found: Vec::new(),
                pan_found_per_subfiles: Vec::new(),
            }),
        };

        match result {
            Ok(res) => {
                analyse_result.nb_found_pan += res.pan_found.len() as u64;
                for entry in &res.pan_found_per_subfiles {
                    analyse_result.nb_found_pan += entry.pan_found.len() as u64;
                }
                analyse_result.results_list.push(res);
            }
            Err(error_msg) => {
                analyse_result.nb_error += 1;
                analyse_result.results_list.push(FileAnalyseResult {
                    filename,
                    error_msg,
                    pan_found: Vec::new(),
                    pan_found_per_subfiles: Vec::new(),
                });
            }
        }
    }
    progress_bar.finish();

    analyse_result
}

fn init_progress_bar(quiet_mode: bool, nb_file: u64) -> ProgressBar {
    let progress_bar = if quiet_mode {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(nb_file)
    };
    progress_bar.set_prefix("Analyse files");
    progress_bar.set_style(
        ProgressStyle::with_template("{prefix:.bold.dim} {wide_bar} {pos}/{len}").unwrap(),
    );

    progress_bar
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
