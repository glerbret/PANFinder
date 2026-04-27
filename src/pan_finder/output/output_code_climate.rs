use chrono::{DateTime, Local};
use hmac_sha512::Hash;
use serde_json::json;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Error;
use std::io::Write;

use crate::pan_finder::analyser::{AnalyseResult, FileAnalyseResult};
use crate::pan_finder::config::Configuration;

pub fn output_code_climate(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<(), Error> {
    let mut file = build_filename(analyse_datetime, config)?;

    let mut vector: Vec<serde_json::Value> = Vec::new();
    for item in &result.results_list {
        if !item.error_msg.is_empty() {
            vector.push(build_error_record(item));
        }

        if !item.pan_found.is_empty() {
            vector.append(&mut build_found_pan_record(item));
        }
    }

    writeln!(file, "{}", json!(vector))?;

    Ok(())
}

fn build_filename(
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<File, Error> {
    let filename = if config.code_climate_filename.is_empty() {
        &format!("PANFinder_{}.json", analyse_datetime.format("%Y%m%d%H%M%S"))
    } else {
        &config.code_climate_filename
    };
    OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(filename)
}

fn build_error_record(item: &FileAnalyseResult) -> serde_json::Value {
    let mut hasher = Hash::new();
    hasher.update(&item.filename);
    hasher.update(&item.error_msg);
    let hash = hasher.finalize();
    json!({
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
    })
}

fn build_found_pan_record(item: &FileAnalyseResult) -> Vec<serde_json::Value> {
    let mut vector: Vec<serde_json::Value> = Vec::new();
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
    vector
}
