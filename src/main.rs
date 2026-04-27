mod pan_finder;
use crate::pan_finder::analyser::analyse_files;
use crate::pan_finder::config::get_config;
use crate::pan_finder::lister::get_files_list;
use crate::pan_finder::output::output_result;

fn main() {
    let analyse_datetime = chrono::offset::Local::now();
    let config = get_config();
    let files_list = get_files_list(&config);
    let result = analyse_files(files_list, &config);
    output_result(&result, analyse_datetime, &config);
}
