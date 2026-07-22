use bitflags::bitflags;
pub use buffer_search::BufferSearchBar;
pub use editor::HighlightKey;
use editor::SearchSettings;
use gpui::{Action, App, ClickEvent, Entity, FocusHandle, IntoElement, actions};
use project::search::SearchQuery;
pub use project_search::ProjectSearchView;
use ui::{IconButtonShape, Tooltip, prelude::*};
use util::paths::PathMatcher;
use workspace::notifications::NotificationId;
use workspace::{Toast, Workspace};
pub use zed_actions::search::ToggleIncludeIgnored;
use zed_i18n::t;

pub use search_status_button::SEARCH_ICON;

use crate::project_search::ProjectSearchBar;

pub mod buffer_search;
pub mod project_search;
pub(crate) mod search_bar;
pub mod search_status_button;
pub mod text_finder;

pub fn init(cx: &mut App) {
    menu::init();
    buffer_search::init(cx);
    project_search::init(cx);
    text_finder::init(cx);
}

actions!(
    search,
    [
        /// Focuses on the search input field.
        FocusSearch,
        /// Toggles whole word matching.
        ToggleWholeWord,
        /// Toggles case-sensitive search.
        ToggleCaseSensitive,
        /// Toggles regular expression mode.
        ToggleRegex,
        /// Toggles the replace interface.
        ToggleReplace,
        /// Toggles searching within selection only.
        ToggleSelection,
        /// Selects the next search match.
        SelectNextMatch,
        /// Selects the previous search match.
        SelectPreviousMatch,
        /// Selects all search matches.
        SelectAllMatches,
        /// Cycles through search modes.
        CycleMode,
        /// Navigates to the next query in search history.
        NextHistoryQuery,
        /// Navigates to the previous query in search history.
        PreviousHistoryQuery,
        /// Replaces all matches.
        ReplaceAll,
        /// Replaces the next match.
        ReplaceNext,
    ]
);

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    pub struct SearchOptions: u8 {
        const NONE = 0;
        const WHOLE_WORD = 1 << SearchOption::WholeWord as u8;
        const CASE_SENSITIVE = 1 << SearchOption::CaseSensitive as u8;
        const INCLUDE_IGNORED = 1 << SearchOption::IncludeIgnored as u8;
        const REGEX = 1 << SearchOption::Regex as u8;
        const ONE_MATCH_PER_LINE = 1 << SearchOption::OneMatchPerLine as u8;
        /// If set, reverse direction when finding the active match
        const BACKWARDS = 1 << SearchOption::Backwards as u8;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SearchOption {
    WholeWord = 0,
    CaseSensitive,
    IncludeIgnored,
    Regex,
    OneMatchPerLine,
    Backwards,
}

// 项目搜索占位符文本经 t!() 在调用点解析(返回 String),
// 故不再以 &'static str 常量定义,对应 key 见 locale 文件 search.placeholder.*。
// Project search placeholders are resolved via t!() at the call sites
// (returning String); see the `search.placeholder.*` keys in the locale files.

pub enum SearchSource<'a, 'b> {
    Buffer,
    Project(&'a Context<'b, ProjectSearchBar>),
}

impl SearchOption {
    pub fn as_options(&self) -> SearchOptions {
        SearchOptions::from_bits(1 << *self as u8).unwrap()
    }

    pub fn label(&self) -> &'static str {
        match self {
            SearchOption::WholeWord => "Match Whole Words",
            SearchOption::CaseSensitive => "Match Case Sensitivity",
            SearchOption::IncludeIgnored => "Also search files ignored by configuration",
            SearchOption::Regex => "Use Regular Expressions",
            SearchOption::OneMatchPerLine => "One Match Per Line",
            SearchOption::Backwards => "Search Backwards",
        }
    }

    /// 用户可见的本地化标签文本,用于 tooltip 等显示场景。
    /// Localized label text for display (tooltips etc.).
    /// 注意:`label()` 返回 &'static str,用于元素 id 等需要静态字符串的位置,
    /// 保持英文原值不变;显示文本一律走本方法。
    /// NOTE: `label()` still returns `&'static str` for positions that require
    /// a static string (e.g. element ids) and intentionally stays in English.
    pub fn label_text(&self) -> String {
        match self {
            SearchOption::WholeWord => t!("search.option.whole_word"),
            SearchOption::CaseSensitive => t!("search.option.case_sensitive"),
            SearchOption::IncludeIgnored => t!("search.option.include_ignored"),
            SearchOption::Regex => t!("search.option.regex"),
            SearchOption::OneMatchPerLine => t!("search.option.one_match_per_line"),
            SearchOption::Backwards => t!("search.option.backwards"),
        }
    }

    pub fn icon(&self) -> ui::IconName {
        match self {
            SearchOption::WholeWord => IconName::WholeWord,
            SearchOption::CaseSensitive => IconName::CaseSensitive,
            SearchOption::IncludeIgnored => IconName::FileIgnored,
            SearchOption::Regex => IconName::Regex,
            _ => panic!("{self:?} is not a named SearchOption"),
        }
    }

    pub fn to_toggle_action(self) -> &'static dyn Action {
        match self {
            SearchOption::WholeWord => &ToggleWholeWord,
            SearchOption::CaseSensitive => &ToggleCaseSensitive,
            SearchOption::IncludeIgnored => &ToggleIncludeIgnored,
            SearchOption::Regex => &ToggleRegex,
            _ => panic!("{self:?} is not a toggle action"),
        }
    }

    pub fn as_button(
        &self,
        active: SearchOptions,
        search_source: SearchSource,
        focus_handle: FocusHandle,
    ) -> impl IntoElement {
        let action = self.to_toggle_action();
        let label = self.label();
        // tooltip 显示本地化文本;label(英文)仅用于元素 id
        // tooltip shows the localized text; the English label is only used as element id
        let tooltip_label = self.label_text();

        IconButton::new(
            (label, matches!(search_source, SearchSource::Buffer) as u32),
            self.icon(),
        )
        .map(|button| match search_source {
            SearchSource::Buffer => {
                let focus_handle = focus_handle.clone();
                button.on_click(move |_: &ClickEvent, window, cx| {
                    if !focus_handle.is_focused(window) {
                        window.focus(&focus_handle, cx);
                    }
                    window.dispatch_action(action.boxed_clone(), cx);
                })
            }
            SearchSource::Project(cx) => {
                let options = self.as_options();
                button.on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.toggle_search_option(options, window, cx);
                }))
            }
        })
        .shape(IconButtonShape::Square)
        .toggle_state(active.contains(self.as_options()))
        .tooltip(move |_window, cx| {
            Tooltip::for_action_in(tooltip_label.clone(), action, &focus_handle, cx)
        })
    }
}

impl SearchOptions {
    pub fn none() -> SearchOptions {
        SearchOptions::NONE
    }

    pub fn from_query(query: &SearchQuery) -> SearchOptions {
        let mut options = SearchOptions::NONE;
        options.set(SearchOptions::WHOLE_WORD, query.whole_word());
        options.set(SearchOptions::CASE_SENSITIVE, query.case_sensitive());
        options.set(SearchOptions::INCLUDE_IGNORED, query.include_ignored());
        options.set(SearchOptions::REGEX, query.is_regex());
        options
    }

    pub fn from_settings(settings: &SearchSettings) -> SearchOptions {
        let mut options = SearchOptions::NONE;
        options.set(SearchOptions::WHOLE_WORD, settings.whole_word);
        options.set(SearchOptions::CASE_SENSITIVE, settings.case_sensitive);
        options.set(SearchOptions::INCLUDE_IGNORED, settings.include_ignored);
        options.set(SearchOptions::REGEX, settings.regex);
        options
    }

    /// Build a [`SearchQuery`] from these options, selecting the regex or text
    /// constructor based on [`SearchOptions::REGEX`]. Inverse of
    /// [`SearchOptions::from_query`].
    pub fn build_query(
        &self,
        query: impl ToString,
        files_to_include: PathMatcher,
        files_to_exclude: PathMatcher,
        match_full_paths: bool,
        buffers: Option<Vec<Entity<language::Buffer>>>,
    ) -> anyhow::Result<SearchQuery> {
        if self.contains(SearchOptions::REGEX) {
            SearchQuery::regex(
                query,
                self.contains(SearchOptions::WHOLE_WORD),
                self.contains(SearchOptions::CASE_SENSITIVE),
                self.contains(SearchOptions::INCLUDE_IGNORED),
                self.contains(SearchOptions::ONE_MATCH_PER_LINE),
                files_to_include,
                files_to_exclude,
                match_full_paths,
                buffers,
            )
        } else {
            SearchQuery::text(
                query,
                self.contains(SearchOptions::WHOLE_WORD),
                self.contains(SearchOptions::CASE_SENSITIVE),
                self.contains(SearchOptions::INCLUDE_IGNORED),
                files_to_include,
                files_to_exclude,
                match_full_paths,
                buffers,
            )
        }
    }
}

pub(crate) fn show_no_more_matches(window: &mut Window, cx: &mut App) {
    window.defer(cx, |window, cx| {
        struct NotifType();
        let notification_id = NotificationId::unique::<NotifType>();

        let Some(workspace) = Workspace::for_window(window, cx) else {
            return;
        };
        workspace.update(cx, |workspace, cx| {
            workspace.show_toast(
                Toast::new(notification_id.clone(), t!("search.toast.no_more_matches")).autohide(),
                cx,
            );
        })
    });
}
