use clap::Parser;
use std::fs;

/// Configuration of application
#[derive(Debug)]
pub struct Configuration {
    pub search_dir: String,
    pub exclusions: Vec<String>,
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
    pub fn new() -> Configuration {
        Configuration {
            search_dir: String::from("."),
            exclusions: Vec::new(),
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

    let mut config = match fs::read_to_string(args.conf_file) {
        Ok(conf_file_content) => {
            let config_from_file: ConfigFile = toml::from_str(conf_file_content.as_str()).unwrap();
            if let Some(parameters) = config_from_file.parameters {
                Configuration {
                    search_dir: parameters.search_dir.unwrap_or(String::from(".")),
                    exclusions: parameters.exclusions.unwrap_or(Vec::new()),
                    report_test_bin: parameters.report_test.unwrap_or(false),
                    check_text: parameters.check_text.unwrap_or(true),
                    check_pdf: parameters.check_pdf.unwrap_or(true),
                    quiet_mode: false,
                    output_console: parameters.output_console.unwrap_or(true),
                    output_text: parameters.output_text.unwrap_or(false),
                    text_filename: parameters.text_filename.unwrap_or(String::new()),
                    output_code_climate: parameters.output_code_climate.unwrap_or(false),
                    code_climate_filename: parameters
                        .code_climate_filename
                        .unwrap_or(String::new()),
                }
            } else {
                Configuration::new()
            }
        }
        Err(_) => Configuration::new(),
    };

    if let Some(search_dir) = args.search_dir {
        config.search_dir = search_dir;
    }
    if let Some(exclusions) = args.exclusions {
        config.exclusions = exclusions
            .split(",")
            .map(|s| s.to_string())
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
    if let Some(text_filename) = args.text_filename {
        config.text_filename = text_filename;
    }
    if args.code_climate {
        config.output_code_climate = true;
    }
    if let Some(codeclimate_filename) = args.code_climate_filename {
        config.code_climate_filename = codeclimate_filename;
    }
    if args.disable_text_check {
        config.check_text = false;
    }
    if args.disable_pdf_check {
        config.check_pdf = false;
    }

    config
}

/// Configuration from command line
#[derive(Parser, Debug)]
#[command(version, about, long_about = None, after_help = "TOML configuration file can provide parameters inside `parameters` section
- `search_dir`: name of directory to analyse
- `exclusion`: list of files and directories exclusion, can be a full path or only part of it (e.g. `.git` to ignore all `.git` subdirectories)
- `report_test`: report found PAN identified as test card
- `check_text`: enable analyse of text file
- `check_pdf`: enable analyse of PDF file
- `output_console`: enable report on console
- `output_text`: enable report in text file
- `text_filename`: name of output file text
- `output_code_climate`: enable report in Code Climate file
- `code_climate_filename`: name of output Code Climate file

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

/// `Parameters` section of configuration file
#[derive(Debug, serde::Deserialize)]
struct ParametersConfigFile {
    search_dir: Option<String>,
    exclusions: Option<Vec<String>>,
    report_test: Option<bool>,
    check_text: Option<bool>,
    check_pdf: Option<bool>,
    output_console: Option<bool>,
    output_text: Option<bool>,
    text_filename: Option<String>,
    output_code_climate: Option<bool>,
    code_climate_filename: Option<String>,
}

/// Configuration from file
#[derive(Debug, serde::Deserialize)]
struct ConfigFile {
    parameters: Option<ParametersConfigFile>,
}
