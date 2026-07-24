use std::sync::Arc;

use anyhow::{Result, anyhow};
use feature_flags::FeatureFlagAppExt as _;
use gpui::{App, AppContext as _, Entity, Task};
use project::{Project, ProjectEntryId, ProjectPath};
use worktree::LoadedBinaryFile;

use crate::OfficePreviewFeatureFlag;
use crate::docx::docx_to_markdown;
use crate::spreadsheet::{SpreadsheetData, parse_spreadsheet};

/// 支持预览的文档类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfficeDocumentKind {
    /// xlsx / xls / ods 电子表格
    Spreadsheet,
    /// docx 文档
    Document,
}

impl OfficeDocumentKind {
    /// 按小写扩展名判断文档类型；不支持的扩展名返回 `None`，
    /// 文件将回退到默认文本编辑器打开流程。
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "xlsx" | "xls" | "ods" => Some(Self::Spreadsheet),
            "docx" => Some(Self::Document),
            _ => None,
        }
    }
}

/// 解析后的文档内容
#[derive(Clone)]
pub enum OfficeContent {
    Spreadsheet(Arc<SpreadsheetData>),
    /// docx 等文档转换成的 Markdown 文本
    Markdown(Arc<String>),
}

/// 项目侧模型：一个已在后台解析完成的 Office 文档。
///
/// 只读，`is_dirty` 恒为 `false`，不参与保存流程。
pub struct OfficeDocument {
    project_path: ProjectPath,
    entry_id: Option<ProjectEntryId>,
    /// 底层文件句柄，用于读取文件名与绝对路径
    file: Arc<worktree::File>,
    pub kind: OfficeDocumentKind,
    pub content: OfficeContent,
}

impl OfficeDocument {
    pub fn file(&self) -> &Arc<worktree::File> {
        &self.file
    }
}

impl project::ProjectItem for OfficeDocument {
    fn try_open(
        project: &Entity<Project>,
        path: &ProjectPath,
        cx: &mut App,
    ) -> Option<Task<Result<Entity<Self>>>> {
        // feature flag 门控：关闭时直接放行给后续 ProjectItem（文本编辑器）
        if !cx.has_flag::<OfficePreviewFeatureFlag>() {
            return None;
        }

        let ext = path.path.extension()?.to_lowercase();
        let kind = OfficeDocumentKind::from_extension(&ext)?;

        let project = project.clone();
        let path = path.clone();
        let background = cx.background_executor().clone();
        Some(cx.spawn(async move |cx| -> Result<Entity<OfficeDocument>> {
            // 取 worktree 并加载二进制内容
            let load_task = project.update(cx, |project, cx| {
                let worktree = project
                    .worktree_for_id(path.worktree_id, cx)
                    .ok_or_else(|| anyhow!("no such worktree"))?;
                anyhow::Ok(worktree.update(cx, |worktree, cx| {
                    worktree.load_binary_file(path.path.as_ref(), cx)
                }))
            })?;
            let LoadedBinaryFile { file, content } = load_task.await?;

            // 解析放到后台线程，避免大文件阻塞 UI
            let ext_for_parse = ext.clone();
            let content = background
                .spawn(async move {
                    let parsed: Result<OfficeContent> = match kind {
                        OfficeDocumentKind::Spreadsheet => Ok(OfficeContent::Spreadsheet(
                            Arc::new(parse_spreadsheet(content, &ext_for_parse)?),
                        )),
                        OfficeDocumentKind::Document => Ok(OfficeContent::Markdown(Arc::new(
                            docx_to_markdown(content)?,
                        ))),
                    };
                    parsed
                })
                .await?;

            Ok(cx.new(|_| OfficeDocument {
                entry_id: file.entry_id,
                project_path: path,
                file,
                kind,
                content,
            }))
        }))
    }

    fn entry_id(&self, _cx: &App) -> Option<ProjectEntryId> {
        self.entry_id
    }

    fn project_path(&self, _cx: &App) -> Option<ProjectPath> {
        Some(self.project_path.clone())
    }

    fn is_dirty(&self) -> bool {
        false
    }
}
