# PANFinder

This program searches for PAN in files from a root directory

Detected PAN are numbers that:

* Start with a well-known Bank or Financial institution IIN
* Have a length consistent with the IIN (e.g. 16 digits for Visa PAN)
* Have a correct Lühn key number

_Note:_ spaces inside number are ignored for this detection

## Usage

```text
Search for PAN in files

Usage: PANFinder.exe [OPTIONS]

Options:
  -s, --search-dir <SEARCH_DIR>
          Name of directory to analyse [default: .]
  -e, --exclusions <EXCLUSIONS>
          List, join by comma, of excluded files and directory (full path or part of it) [default: empty]
      --no-console
          Disable console report
      --text
          Enable text file report
      --text-filename <TEXT_FILENAME>
          Name of output text file [default: PANFinder_<datetime>.txt]
      --excel
          Enable XLSX file report
      --excel-filename <EXCEL_FILENAME>
          Name of output XLSX file [default: PANFinder_<datetime>.xlsx]
      --code-climate
          Enable Code Climate file report
      --code-climate-filename <CODE_CLIMATE_FILENAME>
          Name of output Code Climate file [default: PANFinder_<datetime>.json]
      --report-test
          Enable report of PAN identified as test card
      --disable-text-check
          Disable analyse of text file
      --disable-pdf-check
          Disable analyse of PDF file
      --disable-tar-check
          Disable analyse of TAR archive
  -q, --quiet-mode
          Quiet mode
  -c, --conf-file <CONF_FILE>
          Name of configuration file [default: ./PANFinder.toml]
  -h, --help
          Print help
  -V, --version
          Print version

Parameters can be provided by a TOML configuration file too
If a parameter is set in both configuration file and command line arguments, the program uses in prior the value in command line arguments
```

## Supported file format

The supported format, with format detection criteria, are listed hereafter

* Text files: files without any `0` in the first 2000 bytes and not yet identified as other file type
* PDF files: files with `%PDF` as first 4 digits
* TAR archives: files with `ustar` as `[257; 262[` bytes

## Configuration file

Program can be parameterized with a TOML configuration file (`./PANFinder.toml` by default)

Parameters live in a `parameters` section:

* `search_dir`: name of directory to analyse
* `report_test`: report found PAN identified as test card
* `check_text`: enable analyse of text file
* `check_pdf`: enable analyse of PDF file
* `check_tar`: enable analyse of TAR archive
* `output_console`: enable report on console
* `output_text`: enable report in text file
* `text_filename`: name of output file text
* `output_excel`: enable report in XLSX file
* `excel_filename`: name of output XLSX file
* `output_code_climate`: enable report in Code Climate file
* `code_climate_filename`: name of output Code Climate file

Exclusion live in a `exclusions` section:

* `path`: list of files and subdirectories (full path or part of path) to ignore in analyse
* `pan`: list of PAN (or beginning of PAN) to exclude
* `pan_<filename>`: list of PAN (or beginning of PAN) to exclude for a given file

_Note:_ `filename` must be enclosed by `"` and use `/` as path separator

## Known limitations

* Code Climate report always use `1` as line number
* Analyse of TAR archive only check for included PDF and text files

## Future evolution

* [ ] Support more file format (archives and compressed files, ...)
* [ ] Excel output

## License

The source code for the site is licensed under the MIT license, which you can find in the `LICENSE` file.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
