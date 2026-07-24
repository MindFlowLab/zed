use std::io::{Cursor, Read, Seek};

use anyhow::Result;
use calamine::{Data, Reader};

/// 单个工作表的数据（单元格统一字符串化）
pub struct SheetData {
    pub name: String,
    pub rows: Vec<Vec<String>>,
}

/// 整个电子表格文档（多工作表）
pub struct SpreadsheetData {
    pub sheets: Vec<SheetData>,
}

/// 单个工作表最大读取行数，超出部分截断，避免超大文件耗尽内存
const MAX_ROWS_PER_SHEET: usize = 100_000;

/// 按扩展名选择解析器，读取全部工作表内容
pub fn parse_spreadsheet(bytes: Vec<u8>, ext: &str) -> Result<SpreadsheetData> {
    let cursor = Cursor::new(bytes);
    match ext {
        "xlsx" => parse_workbook(calamine::Xlsx::new(cursor)?),
        "xls" => parse_workbook(calamine::Xls::new(cursor)?),
        "ods" => parse_workbook(calamine::Ods::new(cursor)?),
        _ => anyhow::bail!("unsupported spreadsheet format: {ext}"),
    }
}

/// 通用工作表解析：遍历所有工作表，转为字符串二维表
fn parse_workbook<RS, W>(mut workbook: W) -> Result<SpreadsheetData>
where
    RS: Read + Seek,
    W: Reader<RS>,
    W::Error: std::fmt::Display,
{
    // 先克隆表名列表，避免后续可变借用 worksheet_range 时冲突
    let names: Vec<String> = workbook
        .sheet_names()
        .iter()
        .map(|name| name.to_string())
        .collect();

    let mut sheets = Vec::with_capacity(names.len());
    for name in names {
        // calamine 0.36 的 worksheet_range 直接返回 Result，表不存在时为 Err
        let range = match workbook.worksheet_range(&name) {
            Ok(range) => range,
            Err(err) => {
                log::warn!("failed to read sheet {name}: {err}");
                continue;
            }
        };

        let mut rows = Vec::with_capacity(range.height().min(MAX_ROWS_PER_SHEET));
        for row in range.rows().take(MAX_ROWS_PER_SHEET) {
            rows.push(row.iter().map(cell_to_string).collect());
        }
        sheets.push(SheetData { name, rows });
    }

    if sheets.is_empty() {
        anyhow::bail!("workbook contains no readable sheets");
    }
    Ok(SpreadsheetData { sheets })
}

/// 单元格值转显示字符串；空单元格返回空串
fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试夹具：三个工作表（Users 含混合类型数据、Empty 空表、Notes 中文内容）
    const TEST_XLSX: &[u8] = include_bytes!("../fixtures/test.xlsx");

    #[test]
    fn test_parse_xlsx_sheets() {
        let data = parse_spreadsheet(TEST_XLSX.to_vec(), "xlsx").unwrap();
        let names: Vec<&str> = data.sheets.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Users", "Empty", "Notes"]);

        // Users 表：表头 + 3 行数据，混合类型均字符串化
        let users = &data.sheets[0];
        assert_eq!(users.rows.len(), 4);
        assert_eq!(users.rows[0], vec!["name", "age", "score", "active"]);
        assert_eq!(users.rows[1][0], "Alice");
        assert_eq!(users.rows[1][1], "30");
        assert_eq!(users.rows[1][2], "95.5");
        assert_eq!(users.rows[1][3], "true");
        // 中文内容保持原样
        assert_eq!(users.rows[3][0], "张三");

        // 空表解析为空行集
        assert!(data.sheets[1].rows.is_empty());

        // Notes 表中文表头
        assert_eq!(data.sheets[2].rows[0], vec!["备注", "数量"]);
    }

    #[test]
    fn test_parse_unsupported_extension() {
        assert!(parse_spreadsheet(Vec::new(), "docx").is_err());
    }

    #[test]
    fn test_parse_corrupted_file() {
        assert!(parse_spreadsheet(vec![1, 2, 3, 4], "xlsx").is_err());
    }
}
