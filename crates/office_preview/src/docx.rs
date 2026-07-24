//! DOCX（OOXML WordprocessingML）→ Markdown 转换器，纯 Rust 实现。
//!
//! docx 是 zip 压缩包，`word/document.xml` 描述正文。本模块用 quick-xml
//! 解析该文件，把标题、段落、粗斜体、换行、表格映射为 Markdown；
//! 图片暂不支持，带编号列表统一按无序列表输出。

use std::io::{Cursor, Read};

use anyhow::{Result, anyhow};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

/// 将 docx 文件字节流转换为 Markdown 文本
pub fn docx_to_markdown(bytes: Vec<u8>) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;
    let mut entry = archive
        .by_name("word/document.xml")
        .map_err(|_| anyhow!("invalid docx: missing word/document.xml"))?;
    let mut xml = String::new();
    entry.read_to_string(&mut xml)?;
    Ok(convert_document_xml(&xml))
}

/// document.xml → Markdown 的状态机转换器
#[derive(Default)]
struct DocxConverter {
    out: String,
    /// 当前段落累积的文本
    para_text: String,
    /// 标题级别，0 表示普通段落
    heading_level: u8,
    /// 段落是否为列表项（w:numPr）
    is_list_item: bool,
    /// 当前 run 的粗体/斜体标记
    bold: bool,
    italic: bool,
    /// 是否正在捕获 w:t 文本
    capture_text: bool,
    /// 表格状态
    in_cell: bool,
    cell_paragraphs: Vec<String>,
    current_row: Vec<String>,
    table_rows: Vec<Vec<String>>,
}

impl DocxConverter {
    fn convert(mut self, xml: &str) -> String {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => self.on_start(&e),
                Ok(Event::Empty(e)) => self.on_empty(&e),
                Ok(Event::End(e)) => self.on_end(local_name(e.name().as_ref())),
                Ok(Event::Text(e)) => {
                    if self.capture_text
                        && let Ok(text) = e.decode()
                    {
                        self.push_run_text(&text);
                    }
                }
                Ok(Event::Eof) => break,
                Err(err) => {
                    // 宽松处理：XML 损坏时输出已解析部分而非整体失败
                    log::warn!("docx document.xml parse error: {err}");
                    break;
                }
                _ => {}
            }
        }
        self.out.trim_end().to_string() + "\n"
    }

    fn on_start(&mut self, e: &BytesStart<'_>) {
        match local_name(e.name().as_ref()) {
            b"p" => self.begin_paragraph(),
            b"pStyle" => self.apply_paragraph_style(e),
            b"numPr" => self.is_list_item = true,
            b"r" => {
                self.bold = false;
                self.italic = false;
            }
            b"b" => self.bold = toggle_value(e),
            b"i" => self.italic = toggle_value(e),
            b"t" => self.capture_text = true,
            b"tbl" => self.table_rows.clear(),
            b"tr" => self.current_row.clear(),
            b"tc" => {
                self.in_cell = true;
                self.cell_paragraphs.clear();
            }
            _ => {}
        }
    }

    fn on_empty(&mut self, e: &BytesStart<'_>) {
        match local_name(e.name().as_ref()) {
            // 自闭合的 pStyle/numPr/b/i 同样需要处理
            b"pStyle" => self.apply_paragraph_style(e),
            b"numPr" => self.is_list_item = true,
            b"b" => self.bold = toggle_value(e),
            b"i" => self.italic = toggle_value(e),
            b"br" | b"cr" => self.para_text.push('\n'),
            b"tab" => self.para_text.push('\t'),
            _ => {}
        }
    }

    fn on_end(&mut self, name: &[u8]) {
        match name {
            b"p" => self.flush_paragraph(),
            b"t" => self.capture_text = false,
            b"tc" => {
                self.in_cell = false;
                // 单元格内多段落以空格连接，换行也压平为空格
                let cell = self.cell_paragraphs.join(" ").replace('\n', " ");
                self.current_row.push(cell);
            }
            b"tr" => self.table_rows.push(std::mem::take(&mut self.current_row)),
            b"tbl" => self.emit_table(),
            _ => {}
        }
    }

    fn begin_paragraph(&mut self) {
        self.para_text.clear();
        self.heading_level = 0;
        self.is_list_item = false;
    }

    /// w:pStyle 的 w:val 形如 Heading1 / heading 2 / 标题1 时视为标题
    fn apply_paragraph_style(&mut self, e: &BytesStart<'_>) {
        if let Some(val) = attr_val(e)
            && let Some(level) = heading_level_from_style(&val)
        {
            self.heading_level = level;
        }
    }

    /// run 文本：先转义再按粗斜体包裹
    fn push_run_text(&mut self, raw: &str) {
        if raw.is_empty() {
            return;
        }
        let escaped = escape_markdown(raw);
        match (self.bold, self.italic) {
            (true, true) => self.para_text.push_str(&format!("***{escaped}***")),
            (true, false) => self.para_text.push_str(&format!("**{escaped}**")),
            (false, true) => self.para_text.push_str(&format!("*{escaped}*")),
            (false, false) => self.para_text.push_str(&escaped),
        }
    }

    /// 段落结束：按标题/列表/普通段落输出；单元格内暂存
    fn flush_paragraph(&mut self) {
        let text = std::mem::take(&mut self.para_text);
        if self.heading_level == 0 && !self.is_list_item && text.trim().is_empty() {
            // 空段落仅用于排版间距，直接跳过
            return;
        }
        let line = if self.heading_level > 0 {
            format!("{} {text}", "#".repeat(self.heading_level as usize))
        } else if self.is_list_item {
            format!("- {text}")
        } else {
            text
        };
        if self.in_cell {
            self.cell_paragraphs.push(line);
        } else {
            self.out.push_str(&line);
            self.out.push_str("\n\n");
        }
    }

    /// 输出 Markdown 表格：首行为表头，列数按最宽行补齐
    fn emit_table(&mut self) {
        let rows = std::mem::take(&mut self.table_rows);
        let col_count = rows.iter().map(|row| row.len()).max().unwrap_or(0);
        if col_count == 0 {
            return;
        }

        let push_row = |out: &mut String, row: &[String]| {
            out.push_str("| ");
            for (i, cell) in row
                .iter()
                .chain(std::iter::repeat(&String::new()))
                .take(col_count)
                .enumerate()
            {
                if i > 0 {
                    out.push_str(" | ");
                }
                out.push_str(&escape_table_cell(cell));
            }
            out.push_str(" |\n");
        };

        push_row(&mut self.out, &rows[0]);
        self.out.push_str("|");
        for _ in 0..col_count {
            self.out.push_str(" --- |");
        }
        self.out.push('\n');
        for row in &rows[1..] {
            push_row(&mut self.out, row);
        }
        self.out.push('\n');
    }
}

/// 将 document.xml 转换为 Markdown
fn convert_document_xml(xml: &str) -> String {
    DocxConverter::default().convert(xml)
}

/// 取元素的 w:val（或无前缀 val）属性值
fn attr_val(e: &BytesStart<'_>) -> Option<String> {
    e.attributes()
        .filter_map(|attr| attr.ok())
        .find(|attr| matches!(attr.key.as_ref(), b"w:val" | b"val"))
        .and_then(|attr| String::from_utf8(attr.value.to_vec()).ok())
}

/// 切换类属性（w:b / w:i）：val 为 0/false 时表示关闭，缺省为开启
fn toggle_value(e: &BytesStart<'_>) -> bool {
    match attr_val(e) {
        Some(val) => !matches!(val.as_str(), "0" | "false"),
        None => true,
    }
}

/// 去掉命名空间前缀，返回本地名（w:p → p）
fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|&b| b == b':').next().unwrap_or(name)
}

/// 从样式名推断标题级别：Heading1 / heading 2 / 标题1 → 1..6
fn heading_level_from_style(val: &str) -> Option<u8> {
    let lower = val.to_lowercase();
    if !(lower.starts_with("heading") || lower.starts_with("标题")) {
        return None;
    }
    let digits: String = lower.chars().filter(|c| c.is_ascii_digit()).collect();
    digits
        .parse::<u8>()
        .ok()
        .filter(|&level| (1..=6).contains(&level))
}

/// 转义 Markdown 特殊字符，避免文档内容被误解析为语法
fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '[' | ']' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// 表格单元格转义：竖线加反斜杠，换行压平为空格
fn escape_table_cell(text: &str) -> String {
    text.replace('\n', " ").replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::build_zip;

    /// 构造最小可用 docx（仅 word/document.xml 一个条目）
    fn build_test_docx(document_xml: &str) -> Vec<u8> {
        build_zip(&[("word/document.xml", document_xml)])
    }

    const DOCUMENT_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body>
<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>标题一</w:t></w:r></w:p>
<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>加粗</w:t></w:r><w:r><w:t xml:space="preserve"> 普通 </w:t></w:r><w:r><w:rPr><w:i/></w:rPr><w:t>斜体</w:t></w:r></w:p>
<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>列表项</w:t></w:r></w:p>
<w:tbl>
<w:tr><w:tc><w:p><w:r><w:t>甲</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>乙</w:t></w:r></w:p></w:tc></w:tr>
<w:tr><w:tc><w:p><w:r><w:t>1</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>2|3</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>
<w:p><w:r><w:t>带*号</w:t></w:r><w:r><w:t>的文本</w:t></w:r></w:p>
</w:body>
</w:document>"#;

    #[test]
    fn test_docx_to_markdown() {
        let md = docx_to_markdown(build_test_docx(DOCUMENT_XML)).unwrap();
        assert!(md.contains("# 标题一"), "heading missing:\n{md}");
        assert!(md.contains("**加粗** 普通 *斜体*"), "runs missing:\n{md}");
        assert!(md.contains("- 列表项"), "list item missing:\n{md}");
        assert!(md.contains("| 甲 | 乙 |"), "table header missing:\n{md}");
        assert!(
            md.contains("| --- | --- |"),
            "table separator missing:\n{md}"
        );
        // 单元格内竖线被转义
        assert!(md.contains("| 1 | 2\\|3 |"), "cell escape missing:\n{md}");
        // 正文星号被转义，不会变成斜体语法
        assert!(md.contains("带\\*号的文本"), "text escape missing:\n{md}");
    }

    #[test]
    fn test_heading_style_variants() {
        assert_eq!(heading_level_from_style("Heading1"), Some(1));
        assert_eq!(heading_level_from_style("heading 3"), Some(3));
        assert_eq!(heading_level_from_style("标题2"), Some(2));
        assert_eq!(heading_level_from_style("Normal"), None);
        assert_eq!(heading_level_from_style("Heading9"), None);
    }

    #[test]
    fn test_missing_document_xml() {
        let bytes = build_zip(&[(
            "mimetype",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        )]);
        assert!(docx_to_markdown(bytes).is_err());
    }

    #[test]
    fn test_not_a_zip() {
        assert!(docx_to_markdown(vec![1, 2, 3, 4]).is_err());
    }
}
