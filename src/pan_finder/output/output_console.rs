use chrono::{DateTime, Local};
use colored::Colorize;

use crate::pan_finder::analyser::AnalyseResult;
use crate::pan_finder::config::Configuration;

pub fn output_console(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) {
    output_header(result, analyse_datetime, config);
    output_error(result);
    output_pan(result);
    println!("================================================================================");
}

fn output_header(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) {
    println!("================================================================================");
    println!(
        "Result of analyse of \"{}\" at {}",
        config.search_dir,
        analyse_datetime.format("%Y-%m-%d %H:%M:%S")
    );
    println!("================================================================================");
    println!();

    println!("{} files analyzed", result.nb_analyzed_file);
    println!();
}

fn output_error(result: &AnalyseResult) {
    if result.nb_error == 0 {
        println!("{}", "No analyse error".green());
    } else {
        println!(
            "{}",
            format!("{} analyse errors", result.nb_error).bright_red()
        );
        for item in &result.results_list {
            if !item.error_msg.is_empty() {
                println!("  * {}: {}", item.filename, item.error_msg);
            }
        }
    }
    println!();
}
fn output_pan(result: &AnalyseResult) {
    if result.nb_found_pan == 0 {
        println!("{}", "No PAN found".green());
    } else {
        println!(
            "{}",
            format!("{} PAN found", result.nb_found_pan).bright_red()
        );
        for item in &result.results_list {
            if !item.pan_found.is_empty() {
                println!("  * {}:", item.filename);
                for pan in &item.pan_found {
                    println!("    - {}: {}", pan.brand, pan.pan);
                }
            } else if !item.pan_found_per_subfiles.is_empty() {
                println!("  * {}:", item.filename);
                for entry in &item.pan_found_per_subfiles {
                    println!("    * {}:", &entry.subfilename);
                    for pan in &entry.pan_found {
                        println!("      - {}: {}", pan.brand, pan.pan);
                    }
                }
            }
        }
    }
}
