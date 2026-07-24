use file_icons::FileIcons;
use gpui::{
    AnyElement, App, Context, Div, Entity, EntityId, EventEmitter, FocusHandle, Focusable,
    FontWeight, IntoElement, ParentElement, Pixels, Render, StatefulInteractiveElement, Styled,
    Task, Window, div, img, px, uniform_list,
};
use language::{File as _, LocalFile as _};
use markdown::{
    CodeBlockRenderer, CopyButtonVisibility, Markdown, MarkdownElement, MarkdownFont,
    MarkdownStyle, WrapButtonVisibility,
};
use settings::Settings as _;
use ui::{TintColor, Tooltip, prelude::*};
use util::paths::PathExt;
use workspace::item::{Item, ProjectItem as WorkspaceProjectItem, TabContentParams};
use workspace::{ItemSettings, Pane, WorkspaceId};
use zed_i18n::t;

use crate::document::{OfficeContent, OfficeDocument};
use crate::pdf::PdfData;
use crate::spreadsheet::SpreadsheetData;
use crate::{ResetZoom, ZoomIn, ZoomOut};

/// 单元格固定宽度
const CELL_WIDTH: Pixels = px(140.0);
/// 行号列宽度
const ROW_NUMBER_WIDTH: Pixels = px(56.0);
/// 表格行高
const ROW_HEIGHT: Pixels = px(24.0);
/// PDF 页面基准显示宽度（zoom = 1.0 时）
const PAGE_DISPLAY_WIDTH: f32 = 800.0;
/// PDF 缩放步长与上下限
const ZOOM_STEP: f32 = 1.25;
const MIN_ZOOM: f32 = 0.25;
const MAX_ZOOM: f32 = 4.0;

/// Office 文档只读预览视图
pub struct OfficePreviewView {
    document: Entity<OfficeDocument>,
    focus_handle: FocusHandle,
    /// 当前激活的工作表索引（仅电子表格使用）
    active_sheet: usize,
    /// docx 等文档的 Markdown 渲染实体（构造时按内容创建）
    markdown: Option<Entity<Markdown>>,
    /// PDF 显示缩放倍率（1.0 = 基准宽度 800px）
    pdf_zoom: f32,
}

impl OfficePreviewView {
    pub fn new(document: Entity<OfficeDocument>, cx: &mut Context<Self>) -> Self {
        // 文档类内容在构造时创建 Markdown 实体（其内部异步解析文本）
        let markdown = match &document.read(cx).content {
            OfficeContent::Markdown(text) => {
                let text: SharedString = text.as_str().into();
                Some(cx.new(|cx| Markdown::new(text, None, None, cx)))
            }
            _ => None,
        };
        Self {
            focus_handle: cx.focus_handle(),
            document,
            active_sheet: 0,
            markdown,
            pdf_zoom: 1.0,
        }
    }

    /// 渲染电子表格：工作表标签栏 + 表头 + 虚拟化数据行
    fn render_spreadsheet(&self, data: &SpreadsheetData, cx: &mut Context<Self>) -> AnyElement {
        let active = self.active_sheet.min(data.sheets.len().saturating_sub(1));

        // 工作表切换标签栏
        let tabs = h_flex()
            .id("office-preview-sheet-tabs")
            .flex_none()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .overflow_x_scroll()
            .children(data.sheets.iter().enumerate().map(|(index, sheet)| {
                Button::new(("sheet-tab", index), sheet.name.clone())
                    .size(ButtonSize::Compact)
                    .toggle_state(index == active)
                    .selected_style(ButtonStyle::Tinted(TintColor::Accent))
                    .tooltip(Tooltip::text(t!(
                        "office_preview.sheet_tab_tooltip",
                        name = sheet.name.clone()
                    )))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_sheet = index;
                        cx.notify();
                    }))
            }));

        let Some(sheet) = data.sheets.get(active) else {
            return v_flex()
                .size_full()
                .child(tabs)
                .child(Self::render_centered_message(
                    t!("office_preview.empty_sheet").into(),
                ))
                .into_any_element();
        };

        if sheet.rows.is_empty() {
            return v_flex()
                .size_full()
                .child(tabs)
                .child(Self::render_centered_message(
                    t!("office_preview.empty_sheet").into(),
                ))
                .into_any_element();
        }

        // 首行作表头，其余为数据行
        let col_count = sheet.rows.iter().map(|row| row.len()).max().unwrap_or(0);
        let table_width = ROW_NUMBER_WIDTH + CELL_WIDTH * col_count as f32;

        let header = render_table_row(&sheet.rows[0], 1, col_count, true, cx);

        let data_row_count = sheet.rows.len() - 1;
        let rows: AnyElement = if data_row_count == 0 {
            Self::render_centered_message(t!("office_preview.empty_sheet").into())
        } else {
            let document = self.document.clone();
            uniform_list(
                "spreadsheet-rows",
                data_row_count,
                move |range, _window, cx| {
                    let doc = document.read(cx);
                    let OfficeContent::Spreadsheet(data) = &doc.content else {
                        return Vec::new();
                    };
                    let Some(sheet) = data.sheets.get(active) else {
                        return Vec::new();
                    };
                    range
                        .map(|display_index| {
                            // display_index 0 对应源数据第 2 行（第 1 行是表头）
                            let row_index = display_index + 1;
                            render_table_row(
                                &sheet.rows[row_index],
                                row_index + 1,
                                col_count,
                                false,
                                cx,
                            )
                        })
                        .collect()
                },
            )
            .flex_1()
            .w(table_width)
            .into_any_element()
        };

        v_flex()
            .id("office-preview")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(tabs)
            .child(
                div()
                    .id("office-preview-table-scroll")
                    .flex_1()
                    .overflow_x_scroll()
                    .child(v_flex().w(table_width).h_full().child(header).child(rows)),
            )
            .into_any_element()
    }

    /// 渲染 docx 等文档：复用 markdown crate 的渲染元素
    fn render_markdown(&self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(markdown) = self.markdown.clone() else {
            return Self::render_centered_message(t!("office_preview.empty_document").into());
        };
        let markdown_style = MarkdownStyle::themed(MarkdownFont::Preview, window, cx);
        div()
            .id("office-preview")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .child(
                div()
                    .id("office-preview-markdown-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .p_4()
                    .child(
                        MarkdownElement::new(markdown, markdown_style).code_block_renderer(
                            CodeBlockRenderer::Default {
                                copy_button_visibility: CopyButtonVisibility::VisibleOnHover,
                                wrap_button_visibility: WrapButtonVisibility::Hidden,
                                border: false,
                            },
                        ),
                    ),
            )
            .into_any_element()
    }

    /// 渲染 PDF：纵向滚动页面列表，缩放动作调整显示尺寸
    fn render_pdf(&self, data: &PdfData, cx: &mut Context<Self>) -> AnyElement {
        let zoom = self.pdf_zoom;
        div()
            .id("office-preview")
            .track_focus(&self.focus_handle)
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .on_action(cx.listener(|this, _: &ZoomIn, _, cx| {
                this.pdf_zoom = (this.pdf_zoom * ZOOM_STEP).min(MAX_ZOOM);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ZoomOut, _, cx| {
                this.pdf_zoom = (this.pdf_zoom / ZOOM_STEP).max(MIN_ZOOM);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ResetZoom, _, cx| {
                this.pdf_zoom = 1.0;
                cx.notify();
            }))
            .child(
                div()
                    .id("office-preview-pdf-scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .py_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .children(data.pages.iter().enumerate().map(|(index, page)| {
                        // 按页面像素比例计算显示高度，缩放统一作用于宽高
                        let width = px(PAGE_DISPLAY_WIDTH * zoom);
                        let height =
                            px(PAGE_DISPLAY_WIDTH * zoom * page.height as f32 / page.width as f32);
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_1()
                            .child(img(page.image.clone()).w(width).h(height))
                            .child(
                                Label::new(t!("office_preview.pdf_page_label", number = index + 1))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                    })),
            )
            .into_any_element()
    }

    /// 空状态等居中提示文本
    fn render_centered_message(message: SharedString) -> AnyElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .child(Label::new(message).color(Color::Muted))
            .into_any_element()
    }
}

/// 渲染一行表格：行号列 + 各数据单元格；不足列数的行补空白格对齐边框
fn render_table_row(
    row: &[String],
    row_number: usize,
    col_count: usize,
    is_header: bool,
    cx: &App,
) -> Div {
    let border_color = cx.theme().colors().border;
    let header_bg = cx.theme().colors().panel_background;

    h_flex()
        .h(ROW_HEIGHT)
        .flex_none()
        // 行号列
        .child(
            div()
                .w(ROW_NUMBER_WIDTH)
                .h_full()
                .flex_none()
                .px_2()
                .bg(header_bg)
                .border_r_1()
                .border_color(border_color)
                .child(
                    Label::new(row_number.to_string())
                        .size(LabelSize::Small)
                        .color(Color::Muted)
                        .single_line(),
                ),
        )
        // 数据单元格
        .children(row.iter().map(|text| {
            div()
                .w(CELL_WIDTH)
                .h_full()
                .flex_none()
                .px_2()
                .border_r_1()
                .border_color(border_color)
                .when(is_header, |div| div.bg(header_bg))
                .child(
                    Label::new(text.clone())
                        .single_line()
                        .color(if is_header {
                            Color::Default
                        } else {
                            Color::Muted
                        })
                        .when(is_header, |label| label.weight(FontWeight::MEDIUM)),
                )
        }))
        // 补齐缺失列，保证边框网格完整
        .children((row.len()..col_count).map(|_| {
            div()
                .w(CELL_WIDTH)
                .h_full()
                .flex_none()
                .border_r_1()
                .border_color(border_color)
                .when(is_header, |div| div.bg(header_bg))
        }))
}

impl EventEmitter<()> for OfficePreviewView {}

impl Render for OfficePreviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 先克隆内容（Arc 廉价克隆），避免持有实体借用与 cx 可变借用冲突
        let content = self.document.read(cx).content.clone();
        match content {
            OfficeContent::Spreadsheet(data) => self.render_spreadsheet(&data, cx),
            OfficeContent::Markdown(_) => self.render_markdown(window, cx),
            OfficeContent::Pdf(data) => self.render_pdf(&data, cx),
        }
    }
}

impl Focusable for OfficePreviewView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Item for OfficePreviewView {
    type Event = ();

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        self.document
            .read(cx)
            .file()
            .file_name(cx)
            .to_string()
            .into()
    }

    fn tab_tooltip_text(&self, cx: &App) -> Option<SharedString> {
        let abs_path = self.document.read(cx).file().abs_path(cx);
        Some(abs_path.compact().to_string_lossy().into_owned().into())
    }

    fn tab_icon(&self, _window: &Window, cx: &App) -> Option<Icon> {
        let path = self.document.read(cx).file().abs_path(cx);
        ItemSettings::get_global(cx)
            .file_icons
            .then(|| FileIcons::get_icon(path.as_ref(), cx))
            .flatten()
            .map(Icon::from_path)
    }

    fn for_each_project_item(
        &self,
        cx: &App,
        f: &mut dyn FnMut(EntityId, &dyn project::ProjectItem),
    ) {
        f(self.document.entity_id(), self.document.read(cx))
    }

    fn clone_on_split(
        &self,
        _workspace_id: Option<WorkspaceId>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Task<Option<Entity<Self>>>
    where
        Self: Sized,
    {
        let document = self.document.clone();
        Task::ready(Some(cx.new(|cx| Self::new(document, cx))))
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(params.detail.unwrap_or_default(), cx))
            .single_line()
            .color(params.text_color())
            .when(params.preview, |label| label.italic())
            .into_any_element()
    }

    /// 只读预览不需要编辑器工具栏
    fn show_toolbar(&self) -> bool {
        false
    }
}

impl WorkspaceProjectItem for OfficePreviewView {
    type Item = OfficeDocument;

    fn for_project_item(
        _project: Entity<project::Project>,
        _pane: Option<&Pane>,
        item: Entity<Self::Item>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new(item, cx)
    }
}
