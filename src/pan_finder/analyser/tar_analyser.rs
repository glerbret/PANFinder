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
    let mut results = FileAnalyseResult {
        filename: file.path().to_str().unwrap().to_string(),
        error_msg: String::new(),
        pan_found: Vec::new(),
        pan_found_per_subfiles: Vec::new(),
    };

    match File::open(file.path()) {
        Ok(tar_file) => {
            let mut archive = Archive::new(tar_file);

            let archive_content = match archive.entries() {
                Ok(archive_content) => archive_content,
                Err(e) => {
                    return Err(format!(
                        "read entries error {} {}",
                        file.path().to_str().unwrap(),
                        e
                    ));
                }
            };

            for inc_file in archive_content {
                let mut inc_file = match inc_file {
                    Ok(in_file) => in_file,
                    Err(e) => {
                        return Err(format!(
                            "read embedded file error {} {}",
                            file.path().to_str().unwrap(),
                            e
                        ));
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

        Err(e) => Err(format!(
            "read error {} {}",
            file.path().to_str().unwrap(),
            e
        )),
    }
}

fn check_inc_file(
    patterns_list: &Vec<Pattern>,
    config: &Configuration,
    inc_file: &mut Entry<'_, File>,
) -> Result<Vec<PanFound>, String> {
    let size = usize::try_from(inc_file.header().size().unwrap()).unwrap();
    let mut data = vec![0u8; size];

    match read_up_to(inc_file, &mut data) {
        Ok(_) => {
            let filename = inc_file.header().path().unwrap();
            if is_pdf_file(&data, size) {
                check_pdf_file(patterns_list, config, filename.to_str().unwrap(), data)
            } else if is_text_file(&data, size) {
                check_test_file(patterns_list, config, filename.to_str().unwrap(), &data)
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

fn check_test_file(
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
