//! Office 文档与 PDF 只读预览（xlsx/xls/ods，后续扩展 docx/pptx/pdf）。
//!
//! 仿照 `image_viewer` 的 ProjectItem 分发模式：注册后按扩展名拦截文件打开流程，
//! 后台加载并解析文件，渲染为只读预览视图。整个功能由 `office-preview`
//! feature flag 门控。

mod document;
mod docx;
mod markup;
mod pdf;
mod pptx;
mod spreadsheet;
mod view;

pub use document::{OfficeContent, OfficeDocument, OfficeDocumentKind};
pub use docx::docx_to_markdown;
pub use pdf::{PdfData, PdfPage};
pub use pptx::pptx_to_markdown;
pub use spreadsheet::{SheetData, SpreadsheetData};
pub use view::OfficePreviewView;

use feature_flags::{FeatureFlag, PresenceFlag, register_feature_flag};
use gpui::{App, actions};

actions!(
    office_preview,
    [
        /// PDF 预览放大。
        ZoomIn,
        /// PDF 预览缩小。
        ZoomOut,
        /// PDF 预览重置缩放。
        ResetZoom
    ]
);

/// 门控 Office/PDF 预览功能的 feature flag。
///
/// 本 fork 通过 `enabled_for_all` 默认对所有用户开启；
/// 如需整体停用该功能，把该方法改为返回 `false` 即可。
pub struct OfficePreviewFeatureFlag;

impl FeatureFlag for OfficePreviewFeatureFlag {
    const NAME: &'static str = "office-preview";
    type Value = PresenceFlag;

    fn enabled_for_all() -> bool {
        true
    }
}
register_feature_flag!(OfficePreviewFeatureFlag);

pub fn init(cx: &mut App) {
    workspace::register_project_item::<OfficePreviewView>(cx);
}

/// 测试专用：在内存中构建 zip 压缩包（docx/xlsx 等 OOXML 夹具）
#[cfg(test)]
pub(crate) mod test_util {
    use std::io::{Cursor, Write};

    /// 按 (条目名, 内容) 列表构建 deflate 压缩的 zip 字节流
    pub fn build_zip(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for (name, content) in entries {
                zip.start_file(*name, options).unwrap();
                zip.write_all(content.as_bytes()).unwrap();
            }
            zip.finish().unwrap();
        }
        buf.into_inner()
    }
}
