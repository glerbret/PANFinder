mod pan_finder;
use crate::pan_finder::analyser::*;
use crate::pan_finder::config::*;
use crate::pan_finder::lister::*;
use crate::pan_finder::output::*;

fn main() {
    let analyse_datetime = chrono::offset::Local::now();
    let config = get_config();
    let files_list = get_files_list(&config);
    let result = analyse_files(files_list, &config);
    output_result(result, analyse_datetime, &config);
}
