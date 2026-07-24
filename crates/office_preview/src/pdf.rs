//! PDF → 逐页位图，纯 Rust 实现（pdf-render / vello_cpu）。
//!
//! 打开时后台把每一页光栅化为 RGBA 位图（`RenderImage`），
//! 视图层以滚动列表展示；缩放通过调整显示尺寸实现。

use std::sync::Arc;

use anyhow::Result;
use gpui::RenderImage;
use pdf_render::RenderSettings;
use pdf_render::pdf_interpret::InterpreterSettings;
use pdf_render::pdf_syntax::Pdf;
use pdf_render::vello_cpu::color::palette::css::WHITE;

/// 单页渲染结果
pub struct PdfPage {
    pub image: Arc<RenderImage>,
    /// 位图像素宽（渲染缩放下）
    pub width: u32,
    /// 位图像素高（渲染缩放下）
    pub height: u32,
}

/// 整个 PDF 文档的渲染数据
pub struct PdfData {
    pub pages: Vec<PdfPage>,
}

/// 最大渲染页数，超出截断，避免超大文档耗尽内存
pub const MAX_PDF_PAGES: usize = 500;

/// 默认渲染缩放：PDF 基准 72dpi，1.5 ≈ 108dpi，兼顾清晰度与内存
pub const DEFAULT_SCALE: f32 = 1.5;

/// 按指定缩放渲染全部页面（在后台线程调用）
pub fn render_pdf_pages(bytes: Vec<u8>, scale: f32) -> Result<PdfData> {
    let pdf = Pdf::new(bytes).map_err(|err| anyhow::anyhow!("failed to load pdf: {err:?}"))?;
    let settings = InterpreterSettings::default();
    let render_settings = RenderSettings {
        x_scale: scale,
        y_scale: scale,
        bg_color: WHITE,
        ..Default::default()
    };

    let mut pages = Vec::new();
    for page in pdf.pages().iter().take(MAX_PDF_PAGES) {
        let pixmap = pdf_render::render(page, &settings, &render_settings);
        let width = pixmap.width() as u32;
        let height = pixmap.height() as u32;

        // Pixmap 为预乘 alpha，转换为直通 RGBA8 供 GPUI 使用
        let rgba: Vec<u8> = pixmap
            .take_unpremultiplied()
            .into_iter()
            .flat_map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
            .collect();
        let Some(image) = image::RgbaImage::from_raw(width, height, rgba) else {
            anyhow::bail!("failed to construct page image ({width}x{height})");
        };

        pages.push(PdfPage {
            image: Arc::new(RenderImage::new(smallvec::smallvec![image::Frame::new(
                image
            )])),
            width,
            height,
        });
    }

    if pages.is_empty() {
        anyhow::bail!("pdf contains no pages");
    }
    Ok(PdfData { pages })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造单页最小合法 PDF（200x100pt 空白页，xref 偏移精确计算）
    fn build_minimal_pdf() -> Vec<u8> {
        let objects = [
            "1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n",
            "2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n",
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Resources << >> /Contents 4 0 R >>\nendobj\n",
            "4 0 obj\n<< /Length 0 >>\nstream\nendstream\nendobj\n",
        ];

        let mut out = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        for object in &objects {
            offsets.push(out.len());
            out.push_str(object);
        }

        // xref 表与 trailer
        let xref_offset = out.len();
        out.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
        out.push_str("0000000000 65535 f \n");
        for offset in &offsets {
            out.push_str(&format!("{:010} 00000 n \n", offset));
        }
        out.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            objects.len() + 1,
            xref_offset
        ));
        out.into_bytes()
    }

    #[test]
    fn test_render_minimal_pdf() {
        let data = render_pdf_pages(build_minimal_pdf(), 1.0).unwrap();
        assert_eq!(data.pages.len(), 1);
        // MediaBox 200x100pt，缩放 1.0 → 200x100 像素
        assert_eq!(data.pages[0].width, 200);
        assert_eq!(data.pages[0].height, 100);
    }

    #[test]
    fn test_render_scale() {
        let data = render_pdf_pages(build_minimal_pdf(), 2.0).unwrap();
        assert_eq!(data.pages[0].width, 400);
        assert_eq!(data.pages[0].height, 200);
    }

    #[test]
    fn test_render_invalid_pdf() {
        assert!(render_pdf_pages(vec![1, 2, 3, 4], 1.0).is_err());
    }
}
