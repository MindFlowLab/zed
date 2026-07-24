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
    use crate::test_util::build_zip;

    /// 在内存中构建最小可用 xlsx（内联字符串，不依赖 sharedStrings）
    fn build_test_xlsx() -> Vec<u8> {
        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#;
        let rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
        let workbook = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets>
<sheet name="Users" sheetId="1" r:id="rId1"/>
<sheet name="Notes" sheetId="2" r:id="rId2"/>
</sheets>
</workbook>"#;
        let workbook_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
</Relationships>"#;
        let sheet1 = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>name</t></is></c><c r="B1" t="inlineStr"><is><t>age</t></is></c><c r="C1" t="inlineStr"><is><t>score</t></is></c><c r="D1" t="inlineStr"><is><t>active</t></is></c></row>
<row r="2"><c r="A2" t="inlineStr"><is><t>Alice</t></is></c><c r="B2"><v>30</v></c><c r="C2"><v>95.5</v></c><c r="D2" t="b"><v>1</v></c></row>
<row r="3"><c r="A3" t="inlineStr"><is><t>张三</t></is></c><c r="B3"><v>40</v></c></row>
</sheetData>
</worksheet>"#;
        let sheet2 = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<sheetData>
<row r="1"><c r="A1" t="inlineStr"><is><t>备注</t></is></c><c r="B1" t="inlineStr"><is><t>数量</t></is></c></row>
</sheetData>
</worksheet>"#;
        build_zip(&[
            ("[Content_Types].xml", content_types),
            ("_rels/.rels", rels),
            ("xl/workbook.xml", workbook),
            ("xl/_rels/workbook.xml.rels", workbook_rels),
            ("xl/worksheets/sheet1.xml", sheet1),
            ("xl/worksheets/sheet2.xml", sheet2),
        ])
    }

    #[test]
    fn test_parse_xlsx_sheets() {
        let data = parse_spreadsheet(build_test_xlsx(), "xlsx").unwrap();
        let names: Vec<&str> = data.sheets.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["Users", "Notes"]);

        // Users 表：表头 + 2 行数据，混合类型均字符串化
        let users = &data.sheets[0];
        assert_eq!(users.rows.len(), 3);
        assert_eq!(users.rows[0], vec!["name", "age", "score", "active"]);
        assert_eq!(users.rows[1][0], "Alice");
        assert_eq!(users.rows[1][1], "30");
        assert_eq!(users.rows[1][2], "95.5");
        assert_eq!(users.rows[1][3], "true");
        // 中文内容保持原样
        assert_eq!(users.rows[2][0], "张三");

        // Notes 表中文表头
        assert_eq!(data.sheets[1].rows[0], vec!["备注", "数量"]);
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
