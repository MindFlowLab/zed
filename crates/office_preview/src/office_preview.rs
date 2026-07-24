//! Office 文档与 PDF 只读预览（xlsx/xls/ods，后续扩展 docx/pptx/pdf）。
//!
//! 仿照 `image_viewer` 的 ProjectItem 分发模式：注册后按扩展名拦截文件打开流程，
//! 后台加载并解析文件，渲染为只读预览视图。整个功能由 `office-preview`
//! feature flag 门控。

mod document;
mod spreadsheet;
mod view;

pub use document::{OfficeContent, OfficeDocument, OfficeDocumentKind};
pub use spreadsheet::{SheetData, SpreadsheetData};
pub use view::OfficePreviewView;

use feature_flags::{FeatureFlag, PresenceFlag, register_feature_flag};
use gpui::App;

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
