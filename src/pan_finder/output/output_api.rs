use chrono::*;

use crate::pan_finder::analyser::*;
use crate::pan_finder::config::*;

use crate::pan_finder::output::output_console::*;
use crate::pan_finder::output::output_text::*;
use crate::pan_finder::output::output_code_climate::*;

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
