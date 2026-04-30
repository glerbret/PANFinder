use chrono::{DateTime, Local};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;

use crate::pan_finder::analyser::AnalyseResult;
use crate::pan_finder::config::Configuration;

pub fn output_text(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<(), Error> {
    let mut file = build_filename(analyse_datetime, config)?;
    output_header(result, analyse_datetime, config, &mut file)?;
    output_error(result, &mut file)?;
    output_pan(result, &mut file)?;

    Ok(())
}

fn build_filename(
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<File, Error> {
    let filename = if config.text_filename.is_empty() {
        &format!("PANFinder_{}.txt", analyse_datetime.format("%Y%m%d%H%M%S"))
    } else {
        &config.text_filename
    };
    OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(filename)
}

fn output_header(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
    file: &mut File,
) -> Result<(), Error> {
    writeln!(
        file,
        "================================================================================"
    )?;
    writeln!(
        file,
        "Result of analyse of \"{}\" at {}",
        config.search_dir,
        analyse_datetime.format("%Y-%m-%d %H:%M:%S")
    )?;
    writeln!(
        file,
        "================================================================================"
    )?;
    writeln!(file)?;

    writeln!(file, "{} files analyzed", result.nb_analyzed_file)?;
    writeln!(file)?;

    Ok(())
}

fn output_error(result: &AnalyseResult, file: &mut File) -> Result<(), Error> {
    if result.nb_error == 0 {
        writeln!(file, "No analyse error")?;
    } else {
        writeln!(file, "{} analyse errors", result.nb_error)?;
        for item in &result.results_list {
            if !item.error_msg.is_empty() {
                writeln!(file, "  * {}: {}", item.filename, item.error_msg)?;
            }
        }
    }
    writeln!(file)?;

    Ok(())
}

pub fn output_pan(result: &AnalyseResult, file: &mut File) -> Result<(), Error> {
    if result.nb_found_pan == 0 {
        writeln!(file, "No PAN found")?;
    } else {
        writeln!(file, "{} PAN found", result.nb_found_pan)?;
        for item in &result.results_list {
            if !item.pan_found.is_empty() {
                writeln!(file, "  * {}:", item.filename)?;
                for pan in &item.pan_found {
                    writeln!(file, "    - {}: {}", pan.brand, pan.pan)?;
                }
            }
        }
    }

    Ok(())
}
