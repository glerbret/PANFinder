use chrono::*;
use colored::Colorize;
use hmac_sha512::Hash;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;

use crate::pan_finder::analyser::*;
use crate::pan_finder::config::*;

pub fn output_result(
    result: AnalyseResult,
    analyse_datetime: DateTime<Local>,
    config: &Configuration,
) {
    if config.output_console {
        output_console(&result, &analyse_datetime, config);
    }

    if config.output_text {
        output_text(&result, &analyse_datetime, config)
            .unwrap_or_else(|error| println!("Error writing text file result: {}", error));
    }

    if config.output_code_climate {
        output_code_climate(&result, &analyse_datetime, config)
            .unwrap_or_else(|error| println!("Error writing Code Climate file result: {}", error));
    }
}

fn output_console(
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

    if result.nb_error == 0 {
        println!("{}", "No analyse error".green());
    } else {
        println!(
            "{}",
            format!("{} analyse errors", result.nb_error).bright_red()
        );
        for item in &result.results_list {
            if !item.error_msg.is_empty() {
                println!("  {}: {}", item.filename, item.error_msg);
            }
        }
    }
    println!();

    if result.nb_found_pan == 0 {
        println!("{}", "No PAN found".green());
    } else {
        println!(
            "{}",
            format!("{} PAN found", result.nb_found_pan).bright_red()
        );
        for item in &result.results_list {
            if !item.pan_found.is_empty() {
                println!("  {}:", item.filename);
                for pan in &item.pan_found {
                    println!("    - {}: {}", pan.brand, pan.pan)
                }
            }
        }
    }
    println!("================================================================================");
}

fn output_text(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<(), Error> {
    let filename = if !config.text_filename.is_empty() {
        &config.text_filename
    } else {
        &format!("PANFinder_{}.txt", analyse_datetime.format("%Y%m%d%H%M%S"))
    };
    let mut file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(filename)?;

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

    if result.nb_error == 0 {
        writeln!(file, "No analyse error")?;
    } else {
        writeln!(file, "{} analyse errors", result.nb_error)?;
        for item in &result.results_list {
            if !item.error_msg.is_empty() {
                writeln!(file, "  {}: {}", item.filename, item.error_msg)?;
            }
        }
    }
    writeln!(file)?;

    if result.nb_found_pan == 0 {
        writeln!(file, "No PAN found")?;
    } else {
        writeln!(file, "{} PAN found", result.nb_found_pan)?;
        for item in &result.results_list {
            if !item.pan_found.is_empty() {
                writeln!(file, "  {}:", item.filename)?;
                for pan in &item.pan_found {
                    writeln!(file, "    - {}: {}", pan.brand, pan.pan)?
                }
            }
        }
    }

    Ok(())
}

fn output_code_climate(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<(), Error> {
    let filename = if !config.code_climate_filename.is_empty() {
        &config.code_climate_filename
    } else {
        &format!("PANFinder_{}.json", analyse_datetime.format("%Y%m%d%H%M%S"))
    };
    let mut file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(filename)?;

    let mut vector: Vec<serde_json::Value> = Vec::new();
    for item in &result.results_list {
        if !item.error_msg.is_empty() {
            let mut hasher = Hash::new();
            hasher.update(&item.filename);
            hasher.update(&item.error_msg);
            let hash = hasher.finalize();
            let issue = json!({
                "type": "issue",
                "check_name": "Analyse/Parse error",
                "description": item.error_msg,
                "categories": ["Security"],
                "location": {
                     "path": item.filename,
              "lines": {
                "begin": 1,
                "end": 1,
                }},
                "severity": "critical",
                "fingerprint": hex::encode(hash),
            });
            vector.push(issue);
        }

        if !item.pan_found.is_empty() {
            for pan in &item.pan_found {
                let mut hasher = Hash::new();
                hasher.update(&item.filename);
                hasher.update(&pan.brand);
                hasher.update(&pan.pan);
                let hash = hasher.finalize();
                let issue = json!({
                    "type": "issue",
                    "check_name": "Analyse/PAN found",
                    "description": format!("{}: {}", pan.brand, pan.pan),
                    "categories": ["Security"],
                    "location": {
                         "path": item.filename,
                  "lines": {
                    "begin": 1,
                    "end": 1,
                    }},
                    "severity": "critical",
                    "fingerprint": hex::encode(hash),
                });
                vector.push(issue);
            }
        }
    }
    writeln!(file, "{}", json!(vector))?;

    Ok(())
}
