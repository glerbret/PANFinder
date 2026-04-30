use clap::Parser;
use std::fs;
use toml::Table;

/// Configuration of application
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Configuration {
    pub search_dir: String,
    pub excluded_path: Vec<String>,
    pub report_test_bin: bool,
    pub check_text: bool,
    pub check_pdf: bool,
    pub quiet_mode: bool,
    pub output_console: bool,
    pub output_text: bool,
    pub text_filename: String,
    pub output_code_climate: bool,
    pub code_climate_filename: String,
}
impl Configuration {
    pub fn new() -> Self {
        Self {
            search_dir: String::from("."),
            excluded_path: Vec::new(),
            report_test_bin: false,
            check_text: true,
            check_pdf: true,
            quiet_mode: false,
            output_console: true,
            output_text: false,
            text_filename: String::new(),
            output_code_climate: false,
            code_climate_filename: String::new(),
        }
    }
}

/// Get application configuration
pub fn get_config() -> Configuration {
    let args = Args::parse();

    let mut config = Configuration::new();
    read_configuration_file(&mut config, &args.conf_file);
    overload_conf_cli(&mut config, &args);

    config
}

fn read_configuration_file(config: &mut Configuration, conf_file: &String) {
    if let Ok(conf_file_content) = fs::read_to_string(conf_file) {
        let config_from_file = conf_file_content.parse::<Table>().unwrap();

        if let Some(parameters) = config_from_file.get("parameters") {
            let parameters = parameters.as_table().unwrap();
            if parameters.contains_key("search_dir") {
                config.search_dir = parameters["search_dir"].as_str().unwrap().to_string();
            }
            if parameters.contains_key("exclusions") {
                config.excluded_path = parameters["exclusions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| e.as_str().unwrap().to_string())
                    .collect();
            }
            if parameters.contains_key("report_test") {
                config.report_test_bin = parameters["report_test"].as_bool().unwrap();
            }
            if parameters.contains_key("check_text") {
                config.check_text = parameters["check_text"].as_bool().unwrap();
            }
            if parameters.contains_key("check_pdf") {
                config.check_pdf = parameters["check_pdf"].as_bool().unwrap();
            }
            if parameters.contains_key("output_console") {
                config.output_console = parameters["output_console"].as_bool().unwrap();
            }
            if parameters.contains_key("output_text") {
                config.output_text = parameters["output_text"].as_bool().unwrap();
            }
            if parameters.contains_key("text_filename") {
                config.text_filename = parameters["text_filename"].as_str().unwrap().to_string();
            }
            if parameters.contains_key("output_code_climate") {
                config.output_code_climate = parameters["output_code_climate"].as_bool().unwrap();
            }
            if parameters.contains_key("code_climate_filename") {
                config.code_climate_filename = parameters["code_climate_filename"]
                    .as_str()
                    .unwrap()
                    .to_string();
            }
        }

        if let Some(exclusions) = config_from_file.get("exclusions") {
            let exclusions = exclusions.as_table().unwrap();
            if exclusions.contains_key("path") {
                config.excluded_path = exclusions["path"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| e.as_str().unwrap().to_string())
                    .collect();
            }
        }
    }
}

fn overload_conf_cli(config: &mut Configuration, args: &Args) {
    if let Some(search_dir) = &args.search_dir {
        config.search_dir.clone_from(search_dir);
    }
    if let Some(exclusions) = &args.exclusions {
        config.excluded_path = exclusions
            .split(',')
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>();
    }
    if args.report_test {
        config.report_test_bin = true;
    }
    config.quiet_mode = args.quiet_mode;
    if args.no_console {
        config.output_console = false;
    }
    if args.text {
        config.output_text = true;
    }
    if let Some(text_filename) = &args.text_filename {
        config.text_filename.clone_from(text_filename);
    }
    if args.code_climate {
        config.output_code_climate = true;
    }
    if let Some(code_climate_filename) = &args.code_climate_filename {
        config
            .code_climate_filename
            .clone_from(code_climate_filename);
    }
    if args.disable_text_check {
        config.check_text = false;
    }
    if args.disable_pdf_check {
        config.check_pdf = false;
    }
}

/// Configuration from command line
#[derive(Parser, Debug)]
#[allow(clippy::struct_excessive_bools)]
#[command(version, about, long_about = None, after_help = "Parameters can be provided by a TOML configuration file too
If a parameter is set in both configuration file and command line arguments, the program uses in prior the value in command line arguments")]
struct Args {
    /// Name of directory to analyse [default: .]
    #[arg(short, long)]
    search_dir: Option<String>,

    /// List, join by comma, of excluded files and directory (full path or part of it) [default: empty]
    #[arg(short, long)]
    exclusions: Option<String>,

    /// Disable console report
    #[arg(long, default_value_t = false)]
    no_console: bool,

    /// Enable text file report
    #[arg(long, default_value_t = false)]
    text: bool,

    /// Name of output text file [default: PANFinder_<datetime>.txt]
    #[arg(long)]
    text_filename: Option<String>,

    /// Enable Code Climate file report
    #[arg(long, default_value_t = false)]
    code_climate: bool,

    /// Name of output Code Climate file [default: PANFinder_<datetime>.json]
    #[arg(long)]
    code_climate_filename: Option<String>,

    /// Enable report of PAN identified as test card
    #[arg(long, default_value_t = false)]
    report_test: bool,

    /// Disable analyse of text file
    #[arg(long, default_value_t = false)]
    disable_text_check: bool,

    /// Disable analyse of PDF file
    #[arg(long, default_value_t = false)]
    disable_pdf_check: bool,

    /// Quiet mode
    #[arg(short, long, default_value_t = false)]
    quiet_mode: bool,

    /// Name of configuration file
    #[arg(short, long, default_value_t = String::from("./PANFinder.toml"))]
    conf_file: String,
}
