//! PPTX（OOXML PresentationML）→ Markdown 转换器，纯 Rust 实现。
//!
//! pptx 是 zip 压缩包，幻灯片正文位于 `ppt/slides/slideN.xml`。
//! 本模块按幻灯片序号提取文本，输出以「幻灯片 N」为二级标题的 Markdown：
//! 段落、粗斜体、项目符号列表；图片与动画等暂不支持。

use std::io::{Cursor, Read};

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use zed_i18n::t;

use crate::markup::{escape_markdown, wrap_run_style};

/// 将 pptx 文件字节流转换为 Markdown 文本
pub fn pptx_to_markdown(bytes: Vec<u8>) -> Result<String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))?;

    // 收集所有幻灯片条目并按序号排序（zip 内条目顺序不保证）
    let mut slides: Vec<(usize, String)> = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = entry.name().to_string();
        if let Some(number) = slide_number_from_path(&name) {
            slides.push((number, name));
        }
    }
    slides.sort_by_key(|(number, _)| *number);

    let mut out = String::new();
    for (number, name) in slides {
        let mut xml = String::new();
        archive.by_name(&name)?.read_to_string(&mut xml)?;
        let text = convert_slide_xml(&xml);
        if text.trim().is_empty() {
            // 无文本内容的幻灯片（纯图片页等）跳过
            continue;
        }
        out.push_str(&format!(
            "## {}\n\n",
            t!("office_preview.slide_heading", number = number)
        ));
        out.push_str(&text);
    }

    if out.is_empty() {
        anyhow::bail!("presentation contains no slide text");
    }
    Ok(out.trim_end().to_string() + "\n")
}

/// 从条目路径提取幻灯片序号：ppt/slides/slide3.xml → 3
fn slide_number_from_path(path: &str) -> Option<usize> {
    let rest = path.strip_prefix("ppt/slides/slide")?;
    let number = rest.strip_suffix(".xml")?;
    number.parse().ok()
}

/// 单张幻灯片的文本提取状态机（DrawingML：a:p / a:r / a:t）
#[derive(Default)]
struct SlideConverter {
    out: String,
    para_text: String,
    /// 段落是否带项目符号（a:buChar / a:buAutoNum）
    is_bullet: bool,
    bold: bool,
    italic: bool,
    capture_text: bool,
}

impl SlideConverter {
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
                    log::warn!("pptx slide xml parse error: {err}");
                    break;
                }
                _ => {}
            }
        }
        self.out
    }

    fn on_start(&mut self, e: &BytesStart<'_>) {
        match local_name(e.name().as_ref()) {
            b"p" => {
                self.para_text.clear();
                self.is_bullet = false;
            }
            b"r" => {
                self.bold = false;
                self.italic = false;
            }
            b"rPr" => self.apply_run_props(e),
            b"t" => self.capture_text = true,
            b"buChar" | b"buAutoNum" => self.is_bullet = true,
            _ => {}
        }
    }

    fn on_empty(&mut self, e: &BytesStart<'_>) {
        match local_name(e.name().as_ref()) {
            b"rPr" => self.apply_run_props(e),
            b"buChar" | b"buAutoNum" => self.is_bullet = true,
            b"br" => self.para_text.push('\n'),
            _ => {}
        }
    }

    fn on_end(&mut self, name: &[u8]) {
        match name {
            b"p" => self.flush_paragraph(),
            b"t" => self.capture_text = false,
            _ => {}
        }
    }

    /// a:rPr 的 b/i 属性控制粗斜体（"1" 开启，"0" 关闭）
    fn apply_run_props(&mut self, e: &BytesStart<'_>) {
        if let Some(bold) = bool_attr(e, b"b") {
            self.bold = bold;
        }
        if let Some(italic) = bool_attr(e, b"i") {
            self.italic = italic;
        }
    }

    fn push_run_text(&mut self, raw: &str) {
        if raw.is_empty() {
            return;
        }
        let styled = wrap_run_style(&escape_markdown(raw), self.bold, self.italic);
        self.para_text.push_str(&styled);
    }

    /// 段落结束：列表项紧凑排列，普通段落以空行分隔
    fn flush_paragraph(&mut self) {
        let text = std::mem::take(&mut self.para_text);
        if text.trim().is_empty() {
            return;
        }
        if self.is_bullet {
            self.out.push_str(&format!("- {text}\n"));
        } else {
            self.out.push_str(&format!("{text}\n\n"));
        }
    }
}

/// 将单张幻灯片的 XML 转换为 Markdown 片段
fn convert_slide_xml(xml: &str) -> String {
    SlideConverter::default().convert(xml)
}

/// 去掉命名空间前缀，返回本地名（a:p → p）
fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|&b| b == b':').next().unwrap_or(name)
}

/// 读取布尔属性（值为 "1" 时 true，"0" 时 false，缺省 None）
fn bool_attr(e: &BytesStart<'_>, key: &[u8]) -> Option<bool> {
    e.attributes()
        .filter_map(|attr| attr.ok())
        .find(|attr| attr.key.as_ref() == key)
        .map(|attr| attr.value.as_ref() == b"1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::build_zip;

    fn slide_xml(body: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>{body}</p:spTree></p:cSld>
</p:sld>"#
        )
    }

    fn build_test_pptx(slides: &[&str]) -> Vec<u8> {
        let entries: Vec<(String, String)> = slides
            .iter()
            .enumerate()
            .map(|(i, body)| (format!("ppt/slides/slide{}.xml", i + 1), slide_xml(body)))
            .collect();
        let refs: Vec<(&str, &str)> = entries
            .iter()
            .map(|(name, xml)| (name.as_str(), xml.as_str()))
            .collect();
        build_zip(&refs)
    }

    #[test]
    fn test_pptx_to_markdown() {
        let pptx = build_test_pptx(&[
            r#"<p:sp><p:txBody><a:p><a:r><a:t>首页标题</a:t></a:r></a:p></p:txBody></p:sp>
<p:sp><p:txBody>
<a:p><a:pPr><a:buChar char="•"/></a:pPr><a:r><a:t>条目一</a:t></a:r></a:p>
<a:p><a:pPr><a:buChar char="•"/></a:pPr><a:r><a:rPr b="1"/><a:t>条目二</a:t></a:r></a:p>
</p:txBody></p:sp>"#,
            r#"<p:sp><p:txBody><a:p><a:r><a:rPr i="1"/><a:t>第二页斜体</a:t></a:r></a:p></p:txBody></p:sp>"#,
        ]);

        let md = pptx_to_markdown(pptx).unwrap();

        // 标题用 t! 查询，断言同样走 t! 保证与 locale 无关
        let heading1 = format!("## {}", t!("office_preview.slide_heading", number = 1));
        let heading2 = format!("## {}", t!("office_preview.slide_heading", number = 2));
        assert!(md.contains(&heading1), "slide 1 heading missing:\n{md}");
        assert!(md.contains(&heading2), "slide 2 heading missing:\n{md}");
        assert!(md.contains("首页标题"), "title missing:\n{md}");
        assert!(
            md.contains("- 条目一\n- **条目二**\n"),
            "bullets missing:\n{md}"
        );
        assert!(md.contains("*第二页斜体*"), "italic missing:\n{md}");
    }

    #[test]
    fn test_slide_number_from_path() {
        assert_eq!(slide_number_from_path("ppt/slides/slide1.xml"), Some(1));
        assert_eq!(slide_number_from_path("ppt/slides/slide12.xml"), Some(12));
        // 关系文件与布局文件不应匹配
        assert_eq!(
            slide_number_from_path("ppt/slides/_rels/slide1.xml.rels"),
            None
        );
        assert_eq!(
            slide_number_from_path("ppt/slideLayouts/slideLayout1.xml"),
            None
        );
        assert_eq!(
            slide_number_from_path("ppt/notesSlides/notesSlide1.xml"),
            None
        );
    }

    #[test]
    fn test_empty_presentation() {
        let pptx = build_test_pptx(&[""]);
        assert!(pptx_to_markdown(pptx).is_err());
    }

    #[test]
    fn test_not_a_zip() {
        assert!(pptx_to_markdown(vec![1, 2, 3, 4]).is_err());
    }
}
