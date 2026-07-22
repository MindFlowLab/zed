//! Zed 界面国际化(i18n)支持 crate
//! Zed UI internationalization support crate.
//!
//! 设计要点 / Design notes:
//! - locale 文件位于本 crate 的 `locales/` 目录,由 rust-i18n 在编译期内嵌
//!   Locale TOML files are embedded at compile time via rust-i18n.
//! - 默认语言 zh-CN;缺失的 key 按 zh-CN -> en 链回退,支持增量汉化
//!   Default locale is zh-CN; missing keys fall back to English,
//!   which allows translating the UI incrementally.
//! - key 命名规范:`<crate>.<模块>.<条目>`,例如 `app_menus.file.save`
//!   Key convention: `<crate>.<module>.<item>`.
//!
//! 用法 / Usage:
//! ```ignore
//! use zed_i18n::t;
//!
//! let label = t!("app_menus.file.save");
//! ```

rust_i18n::i18n!("locales", fallback = "en");

use std::sync::Once;

/// 默认界面语言(本分支为中文汉化版)
/// Default UI locale for this localized fork.
pub const DEFAULT_LOCALE: &str = "zh-CN";

static INIT: Once = Once::new();

/// 初始化 i18n,将默认界面语言设置为 zh-CN。
/// 应在应用启动早期(任何 UI 构建之前)调用;重复调用安全,仅首次生效。
/// Initializes the default locale. Safe to call multiple times; only the
/// first call takes effect.
pub fn init() {
    INIT.call_once(|| {
        rust_i18n::set_locale(DEFAULT_LOCALE);
    });
}

/// 运行时查询指定 key 的翻译文本。
/// 按 zh-CN -> en 回退链查找;若仍缺失,rust-i18n 返回 key 本身。
/// 注意:运行时 key 查询返回的文本借用自 key 参数,故统一转为
/// `String` 返回,避免生命周期泄漏到调用方。
/// Runtime translation lookup with the fallback chain zh-CN -> en -> key.
pub fn lookup(key: &str) -> String {
    rust_i18n::t!(key).into_owned()
}

/// 带变量插值的翻译查询;locale 模板中的占位符使用 `%{name}` 格式。
/// Translation lookup with interpolation; templates use `%{name}` placeholders.
pub fn lookup_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = lookup(key);
    for (name, value) in args {
        text = text.replace(&format!("%{{{name}}}"), value);
    }
    text
}

/// i18n 翻译宏,返回 `String`。
/// The i18n translation macro; returns `String`.
///
/// ```ignore
/// use zed_i18n::t;
///
/// // 简单查询 / simple lookup
/// let s = t!("app_menus.file.save");
///
/// // 变量插值 / interpolation(模板: "Line %{line} of %{total}")
/// let s = t!("editor.line_of", line = 42, total = 100);
/// ```
#[macro_export]
macro_rules! t {
    ($key:expr $(,)?) => {
        $crate::lookup($key)
    };
    ($key:expr, $($arg_key:ident = $arg_value:expr),+ $(,)?) => {
        $crate::lookup_with($key, &[$((stringify!($arg_key), &($arg_value).to_string())),+])
    };
}

pub use rust_i18n::{available_locales, locale, set_locale};

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    /// locale 是进程级全局状态,rust-i18n 的 set_locale 影响所有线程;
    /// cargo test 默认并行执行,涉及 locale 切换的测试必须串行,
    /// 否则会因竞态非确定性地失败。
    /// The locale is process-global state: `set_locale` affects every thread,
    /// and cargo test runs tests in parallel by default, so tests that switch
    /// the locale must be serialized or they fail non-deterministically.
    static LOCALE_LOCK: Mutex<()> = Mutex::new(());

    /// 验证 locale 子目录分片文件的后缀命名机制:
    /// rust-i18n 3.1.5 以文件主干的最后一段(如 `foo.en.toml` -> `en`)
    /// 判定 locale,故 `locales/en/foo.en.toml` 会并入 en locale,
    /// `locales/zh-CN/foo.zh-CN.toml` 会并入 zh-CN locale。
    /// Verifies the dotted-suffix naming mechanism for split locale files:
    /// rust-i18n derives the locale from the last segment of the file stem,
    /// so `foo.en.toml` merges into the `en` locale.
    #[test]
    fn suffix_named_files_merge_into_locale() {
        let _guard = LOCALE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::set_locale("en");
        assert_eq!(
            crate::lookup("suffix_test"),
            "Suffix Works",
            "en 后缀分片文件未并入 en locale"
        );
        crate::set_locale("zh-CN");
        assert_eq!(
            crate::lookup("suffix_test"),
            "后缀生效",
            "zh-CN 后缀分片文件未并入 zh-CN locale"
        );
    }

    /// 根 locale 文件的 key 必须仍可正常查询(回归保护)。
    /// Root locale file keys must remain queryable (regression guard).
    #[test]
    fn root_file_keys_resolve() {
        let _guard = LOCALE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::set_locale("zh-CN");
        assert_eq!(crate::lookup("app_menus.file.save"), "保存");
        crate::set_locale("en");
        assert_eq!(crate::lookup("app_menus.file.save"), "Save");
    }
}
