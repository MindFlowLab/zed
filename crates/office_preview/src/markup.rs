//! docx/pptx 共用的 Markdown 文本处理工具

/// 转义 Markdown 特殊字符，避免文档内容被误解析为语法
pub(crate) fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '[' | ']' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// 按粗体/斜体标记包裹已转义的文本
pub(crate) fn wrap_run_style(text: &str, bold: bool, italic: bool) -> String {
    match (bold, italic) {
        (true, true) => format!("***{text}***"),
        (true, false) => format!("**{text}**"),
        (false, true) => format!("*{text}*"),
        (false, false) => text.to_string(),
    }
}
