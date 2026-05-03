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
      --clear-pan
          Use clear PAN in report
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
          Disable analyse of text files
      --disable-pdf-check
          Disable analyse of PDF files
      --disable-tar-check
          Disable analyse of TAR archives
      --disable-compress-check
          Disable analyse of compressed files
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

* PDF files: files with `%PDF` as first 4 digits
* TAR archives: files with `ustar` as `[257; 262[` bytes
* GZIP files: files with `0x1F, 0x8B, 0x08` as first digits
* BZIP2 files: files with `BZh` as first digits
* ZIP files: files with `0x50, 0x4B, 0x03, 0x04` or `0x50, 0x4B, 0x07, 0x08` as first digits
* Text files: files without any `0` in the first 2000 bytes and not yet identified as other file type

_Note:_ `0x50, 0x4B, 0x05, 0x06` that indicate an empty ZIP archive is not used as there is nothing to analyse

## Configuration file

Program can be parameterized with a TOML configuration file (`./PANFinder.toml` by default)

Parameters live in a `parameters` section:

* `search_dir`: name of directory to analyse (default: `.`)
* `report_test`: report found PAN identified as test card (default : false)
* `truncated_pan`: use truncated PAN in report (default : true)
* `check_text`: enable analyse of text files (default : true)
* `check_pdf`: enable analyse of PDF files (default : true)
* `check_tar`: enable analyse of TAR archives (default : true)
* `check_compress`: enable analyse of compressed files (default : true)
* `output_console`: enable report on console (default : true)
* `output_text`: enable report in text file (default : false)
* `text_filename`: name of output file text (default: `PANFinder_<datetime>.txt`)
* `output_excel`: enable report in XLSX file (default : false)
* `excel_filename`: name of output XLSX file (default: `PANFinder_<datetime>.xlsx`)
* `output_code_climate`: enable report in Code Climate file (default : false)
* `code_climate_filename`: name of output Code Climate file (default: `PANFinder_<datetime>.txt`)

Exclusion live in a `exclusions` section:

* `path`: list of files and subdirectories (full path or part of path) to ignore in analyse
* `pan`: list of PAN (or beginning of PAN) to exclude
* `pan_<filename>`: list of PAN (or beginning of PAN) to exclude for a given file

_Note:_ `filename` must be enclosed by `"` and use `/` as path separator

## Known limitations

* Code Climate report always use `1` as line number
* Number are reported without space, `\r`, `\n` or `-` characters
* Analyse of archives and compressed files only check few type of embedded files
  * TAR archives: PDF and text files (not compressed files neither archives)
  * GIZ and BZIP2 files: PDF, TAR and text files (not other compressed files)
  * ZIP files: PDF and text files (not compressed files neither archives)
* Due to internal encoding, PAN with space or line feed may be not detected in XLSX, DOCS, ODS and such files

## License

The source code for the site is licensed under the MIT license, which you can find in the `LICENSE` file.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
