use chrono::{DateTime, Local};
use rust_xlsxwriter::{Color, Format, FormatAlign, FormatBorder, Workbook, XlsxError};
use std::path::Path;

use crate::pan_finder::analyser::AnalyseResult;
use crate::pan_finder::config::Configuration;

pub fn output_excel(
    result: &AnalyseResult,
    analyse_datetime: &DateTime<Local>,
    config: &Configuration,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_background_color(Color::RGB(0x00_7F_7F))
        .set_font_color(Color::White)
        .set_border(FormatBorder::Thin);
    let cell_format = Format::new().set_border(FormatBorder::Thin);

    output_error(result, &mut workbook, &header_format, &cell_format)?;
    output_found_pan(result, &mut workbook, &header_format, &cell_format)?;

    let file = build_filename(analyse_datetime, config);
    workbook.save(Path::new(&file))?;

    Ok(())
}

fn build_filename(analyse_datetime: &DateTime<Local>, config: &Configuration) -> String {
    if config.code_climate_filename.is_empty() {
        format!("PANFinder_{}.xlsx", analyse_datetime.format("%Y%m%d%H%M%S"))
    } else {
        config.code_climate_filename.clone()
    }
}

fn output_error(
    result: &AnalyseResult,
    workbook: &mut Workbook,
    header_format: &Format,
    cell_format: &Format,
) -> Result<(), XlsxError> {
    if result.nb_error != 0 {
        let worksheet_error = workbook.add_worksheet();
        worksheet_error.set_name("Error")?;
        worksheet_error.set_freeze_panes(1, 1)?;
        worksheet_error.autofilter(0, 0, u32::try_from(result.nb_error).unwrap(), 1)?;

        worksheet_error.set_column_width(0, 60)?;
        worksheet_error.set_column_width(1, 60)?;

        worksheet_error.write_with_format(0, 0, "File", header_format)?;
        worksheet_error.write_with_format(0, 1, "Error", header_format)?;

        let mut row = 1;
        for item in &result.results_list {
            if !item.error_msg.is_empty() {
                worksheet_error.write_with_format(row, 0, &item.filename, cell_format)?;
                worksheet_error.write_with_format(row, 1, &item.error_msg, cell_format)?;
                row += 1;
            }
        }
    }

    Ok(())
}

fn output_found_pan(
    result: &AnalyseResult,
    workbook: &mut Workbook,
    header_format: &Format,
    cell_format: &Format,
) -> Result<(), XlsxError> {
    if result.nb_found_pan != 0 {
        let worksheet_pan = workbook.add_worksheet();
        worksheet_pan.set_name("PAN found")?;
        worksheet_pan.set_freeze_panes(1, 1)?;
        worksheet_pan.autofilter(0, 0, u32::try_from(result.nb_found_pan).unwrap(), 3)?;

        worksheet_pan.set_column_width(0, 60)?;
        worksheet_pan.set_column_width(1, 60)?;
        worksheet_pan.set_column_width(2, 60)?;
        worksheet_pan.set_column_width(3, 30)?;

        worksheet_pan.write_with_format(0, 0, "File", header_format)?;
        worksheet_pan.write_with_format(0, 1, "Subfile", header_format)?;
        worksheet_pan.write_with_format(0, 2, "Brand", header_format)?;
        worksheet_pan.write_with_format(0, 3, "PAN", header_format)?;

        let mut row = 1;
        for item in &result.results_list {
            if !item.pan_found.is_empty() {
                for pan in &item.pan_found {
                    worksheet_pan.write_with_format(row, 0, &item.filename, cell_format)?;
                    worksheet_pan.write_with_format(row, 1, "", cell_format)?;
                    worksheet_pan.write_with_format(row, 2, &pan.brand, cell_format)?;
                    worksheet_pan.write_with_format(row, 3, &pan.pan, cell_format)?;
                    row += 1;
                }
            }

            if !item.pan_found_per_subfiles.is_empty() {
                for entry in &item.pan_found_per_subfiles {
                    for pan in &entry.pan_found {
                        worksheet_pan.write_with_format(row, 0, &item.filename, cell_format)?;
                        worksheet_pan.write_with_format(row, 1, &entry.subfilename, cell_format)?;
                        worksheet_pan.write_with_format(row, 2, &pan.brand, cell_format)?;
                        worksheet_pan.write_with_format(row, 3, &pan.pan, cell_format)?;
                        row += 1;
                    }
                }
            }
        }
    }

    Ok(())
}
