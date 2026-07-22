use collab_ui::collab_panel;
use gpui::{App, Menu, MenuItem, OsAction};
use release_channel::ReleaseChannel;
use terminal_view::terminal_panel;
use zed_actions::{Quit, assistant, debug_panel, dev, git_panel, project_panel};
use zed_i18n::t;

/// 构建应用菜单栏。
/// 所有菜单文案均通过 `zed_i18n::t!` 查询,key 位于 locale 文件的
/// `app_menus.*` 命名空间(见 crates/zed_i18n/locales/{en,zh-CN}.toml)。
/// Builds the application menu bar. All labels are localized via
/// `zed_i18n::t!` (keys under the `app_menus.*` namespace).
pub fn app_menus(cx: &mut App) -> Vec<Menu> {
    let mut view_items = vec![
        MenuItem::action(
            t!("app_menus.view.zoom_in"),
            zed_actions::IncreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            t!("app_menus.view.zoom_out"),
            zed_actions::DecreaseBufferFontSize { persist: false },
        ),
        MenuItem::action(
            t!("app_menus.view.reset_zoom"),
            zed_actions::ResetBufferFontSize { persist: false },
        ),
        MenuItem::action(
            t!("app_menus.view.reset_all_zoom"),
            zed_actions::ResetAllZoom { persist: false },
        ),
        MenuItem::separator(),
        MenuItem::action(
            t!("app_menus.view.toggle_left_dock"),
            workspace::ToggleLeftDock,
        ),
        MenuItem::action(
            t!("app_menus.view.toggle_right_dock"),
            workspace::ToggleRightDock,
        ),
        MenuItem::action(
            t!("app_menus.view.toggle_bottom_dock"),
            workspace::ToggleBottomDock,
        ),
        MenuItem::action(
            t!("app_menus.view.toggle_all_docks"),
            workspace::ToggleAllDocks,
        ),
        MenuItem::submenu(Menu {
            name: t!("app_menus.view.editor_layout").into(),
            disabled: false,
            items: vec![
                MenuItem::action(
                    t!("app_menus.view.split_up"),
                    workspace::SplitUp::default(),
                ),
                MenuItem::action(
                    t!("app_menus.view.split_down"),
                    workspace::SplitDown::default(),
                ),
                MenuItem::action(
                    t!("app_menus.view.split_left"),
                    workspace::SplitLeft::default(),
                ),
                MenuItem::action(
                    t!("app_menus.view.split_right"),
                    workspace::SplitRight::default(),
                ),
            ],
        }),
        MenuItem::separator(),
        MenuItem::action(
            t!("app_menus.view.project_panel"),
            project_panel::ToggleFocus,
        ),
        MenuItem::action(
            t!("app_menus.view.outline_panel"),
            outline_panel::ToggleFocus,
        ),
        MenuItem::action(t!("app_menus.view.collab_panel"), collab_panel::ToggleFocus),
        MenuItem::action(t!("app_menus.view.terminal_panel"), terminal_panel::Toggle),
        MenuItem::action(
            t!("app_menus.view.debugger_panel"),
            debug_panel::ToggleFocus,
        ),
        MenuItem::action(t!("app_menus.view.agent_panel"), assistant::ToggleFocus),
        MenuItem::action(t!("app_menus.view.git_panel"), git_panel::ToggleFocus),
        MenuItem::separator(),
        MenuItem::action(t!("app_menus.view.diagnostics"), diagnostics::Deploy),
        MenuItem::separator(),
    ];

    if ReleaseChannel::try_global(cx) == Some(ReleaseChannel::Dev) {
        view_items.push(MenuItem::action(
            t!("app_menus.view.toggle_gpui_inspector"),
            dev::ToggleInspector,
        ));
        view_items.push(MenuItem::separator());
    }

    vec![
        Menu {
            name: t!("app_menus.zed_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(t!("app_menus.zed.about"), zed_actions::About),
                MenuItem::action(
                    t!("app_menus.zed.check_for_updates"),
                    auto_update::Check,
                ),
                MenuItem::separator(),
                MenuItem::submenu(Menu::new(t!("app_menus.zed.settings")).items([
                    MenuItem::action(
                        t!("app_menus.zed.open_settings"),
                        zed_actions::OpenSettings,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_settings_file"),
                        super::OpenSettingsFile,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_project_settings"),
                        zed_actions::OpenProjectSettings,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_project_settings_file"),
                        super::OpenProjectSettingsFile,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_default_settings"),
                        super::OpenDefaultSettings,
                    ),
                    MenuItem::separator(),
                    MenuItem::action(
                        t!("app_menus.zed.open_keymap"),
                        zed_actions::OpenKeymap,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_keymap_file"),
                        zed_actions::OpenKeymapFile,
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.open_default_key_bindings"),
                        zed_actions::OpenDefaultKeymap,
                    ),
                    MenuItem::separator(),
                    MenuItem::action(
                        t!("app_menus.zed.select_theme"),
                        zed_actions::theme_selector::Toggle::default(),
                    ),
                    MenuItem::action(
                        t!("app_menus.zed.select_icon_theme"),
                        zed_actions::icon_theme_selector::Toggle::default(),
                    ),
                ])),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::os_submenu(
                    t!("app_menus.zed.services"),
                    gpui::SystemMenuType::Services,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.zed.extensions"),
                    zed_actions::Extensions::default(),
                ),
                #[cfg(not(target_os = "windows"))]
                MenuItem::action(t!("app_menus.zed.install_cli"), install_cli::InstallCliBinary),
                MenuItem::separator(),
                #[cfg(target_os = "macos")]
                MenuItem::action(t!("app_menus.zed.hide_zed"), super::Hide),
                #[cfg(target_os = "macos")]
                MenuItem::action(t!("app_menus.zed.hide_others"), super::HideOthers),
                #[cfg(target_os = "macos")]
                MenuItem::action(t!("app_menus.zed.show_all"), super::ShowAll),
                MenuItem::separator(),
                MenuItem::action(t!("app_menus.zed.quit"), Quit),
            ],
        },
        Menu {
            name: t!("app_menus.file_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(t!("app_menus.file.new"), workspace::NewFile),
                MenuItem::action(t!("app_menus.file.new_window"), workspace::NewWindow),
                MenuItem::separator(),
                #[cfg(not(target_os = "macos"))]
                MenuItem::action(t!("app_menus.file.open_file"), workspace::OpenFiles),
                MenuItem::action(
                    if cfg!(not(target_os = "macos")) {
                        t!("app_menus.file.open_folder")
                    } else {
                        t!("app_menus.file.open_macos")
                    },
                    workspace::Open::default(),
                ),
                MenuItem::action(
                    t!("app_menus.file.open_recent"),
                    zed_actions::OpenRecent::default(),
                ),
                MenuItem::action(
                    t!("app_menus.file.open_remote"),
                    zed_actions::OpenRemote::default(),
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.file.add_folder_to_project"),
                    workspace::AddFolderToProject,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.file.save"),
                    workspace::Save { save_intent: None },
                ),
                MenuItem::action(t!("app_menus.file.save_as"), workspace::SaveAs),
                MenuItem::action(
                    t!("app_menus.file.save_all"),
                    workspace::SaveAll { save_intent: None },
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.file.close_editor"),
                    workspace::CloseActiveItem {
                        save_intent: None,
                        close_pinned: true,
                    },
                ),
                MenuItem::action(t!("app_menus.file.close_project"), workspace::CloseProject),
                MenuItem::action(t!("app_menus.file.close_window"), workspace::CloseWindow),
            ],
        },
        Menu {
            name: t!("app_menus.edit_title").into(),
            disabled: false,
            items: vec![
                MenuItem::os_action(
                    t!("app_menus.edit.undo"),
                    editor::actions::Undo,
                    OsAction::Undo,
                ),
                MenuItem::os_action(
                    t!("app_menus.edit.redo"),
                    editor::actions::Redo,
                    OsAction::Redo,
                ),
                MenuItem::separator(),
                MenuItem::os_action(t!("app_menus.edit.cut"), editor::actions::Cut, OsAction::Cut),
                MenuItem::os_action(
                    t!("app_menus.edit.copy"),
                    editor::actions::Copy,
                    OsAction::Copy,
                ),
                MenuItem::action(
                    t!("app_menus.edit.copy_and_trim"),
                    editor::actions::CopyAndTrim,
                ),
                MenuItem::os_action(
                    t!("app_menus.edit.paste"),
                    editor::actions::Paste,
                    OsAction::Paste,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.edit.find"),
                    search::buffer_search::Deploy::find(),
                ),
                MenuItem::action(
                    t!("app_menus.edit.find_in_project"),
                    workspace::DeploySearch::default(),
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.edit.toggle_line_comment"),
                    editor::actions::ToggleComments::default(),
                ),
            ],
        },
        Menu {
            name: t!("app_menus.selection_title").into(),
            disabled: false,
            items: vec![
                MenuItem::os_action(
                    t!("app_menus.selection.select_all"),
                    editor::actions::SelectAll,
                    OsAction::SelectAll,
                ),
                MenuItem::action(
                    t!("app_menus.selection.expand_selection"),
                    editor::actions::SelectLargerSyntaxNode,
                ),
                MenuItem::action(
                    t!("app_menus.selection.shrink_selection"),
                    editor::actions::SelectSmallerSyntaxNode,
                ),
                MenuItem::action(
                    t!("app_menus.selection.select_next_sibling"),
                    editor::actions::SelectNextSyntaxNode,
                ),
                MenuItem::action(
                    t!("app_menus.selection.select_previous_sibling"),
                    editor::actions::SelectPreviousSyntaxNode,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.selection.add_cursor_above"),
                    editor::actions::AddSelectionAbove {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    t!("app_menus.selection.add_cursor_below"),
                    editor::actions::AddSelectionBelow {
                        skip_soft_wrap: true,
                    },
                ),
                MenuItem::action(
                    t!("app_menus.selection.select_next_occurrence"),
                    editor::actions::SelectNext {
                        replace_newest: false,
                    },
                ),
                MenuItem::action(
                    t!("app_menus.selection.select_previous_occurrence"),
                    editor::actions::SelectPrevious {
                        replace_newest: false,
                    },
                ),
                MenuItem::action(
                    t!("app_menus.selection.select_all_occurrences"),
                    editor::actions::SelectAllMatches,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.selection.move_line_up"),
                    editor::actions::MoveLineUp,
                ),
                MenuItem::action(
                    t!("app_menus.selection.move_line_down"),
                    editor::actions::MoveLineDown,
                ),
                MenuItem::action(
                    t!("app_menus.selection.duplicate_selection"),
                    editor::actions::DuplicateLineDown,
                ),
            ],
        },
        Menu {
            name: t!("app_menus.view_title").into(),
            disabled: false,
            items: view_items,
        },
        Menu {
            name: t!("app_menus.go_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(t!("app_menus.go.back"), workspace::GoBack),
                MenuItem::action(t!("app_menus.go.forward"), workspace::GoForward),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.go.command_palette"),
                    zed_actions::command_palette::Toggle,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.go.go_to_file"),
                    workspace::ToggleFileFinder::default(),
                ),
                // MenuItem::action("Go to Symbol in Project", project_symbols::Toggle),
                MenuItem::action(
                    t!("app_menus.go.go_to_symbol"),
                    zed_actions::outline::ToggleOutline,
                ),
                MenuItem::action(
                    t!("app_menus.go.go_to_line"),
                    editor::actions::ToggleGoToLine,
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.go.go_to_definition"),
                    editor::actions::GoToDefinition::default(),
                ),
                MenuItem::action(
                    t!("app_menus.go.go_to_declaration"),
                    editor::actions::GoToDeclaration,
                ),
                MenuItem::action(
                    t!("app_menus.go.go_to_type_definition"),
                    editor::actions::GoToTypeDefinition,
                ),
                MenuItem::action(
                    t!("app_menus.go.find_all_references"),
                    editor::actions::FindAllReferences::default(),
                ),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.go.next_problem"),
                    editor::actions::GoToDiagnostic::default(),
                ),
                MenuItem::action(
                    t!("app_menus.go.previous_problem"),
                    editor::actions::GoToPreviousDiagnostic::default(),
                ),
            ],
        },
        Menu {
            name: t!("app_menus.run_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(
                    t!("app_menus.run.spawn_task"),
                    zed_actions::Spawn::ViaModal {
                        reveal_target: None,
                    },
                ),
                MenuItem::action(t!("app_menus.run.start_debugger"), debugger_ui::Start),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.run.edit_tasks_json"),
                    zed_actions::OpenProjectTasks,
                ),
                MenuItem::action(
                    t!("app_menus.run.edit_debug_json"),
                    zed_actions::OpenProjectDebugTasks,
                ),
                MenuItem::separator(),
                MenuItem::action(t!("app_menus.run.continue"), debugger_ui::Continue),
                MenuItem::action(t!("app_menus.run.step_over"), debugger_ui::StepOver),
                MenuItem::action(t!("app_menus.run.step_into"), debugger_ui::StepInto),
                MenuItem::action(t!("app_menus.run.step_out"), debugger_ui::StepOut),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.run.toggle_breakpoint"),
                    editor::actions::ToggleBreakpoint,
                ),
                MenuItem::action(
                    t!("app_menus.run.edit_breakpoint"),
                    editor::actions::EditLogBreakpoint,
                ),
                MenuItem::action(
                    t!("app_menus.run.clear_all_breakpoints"),
                    debugger_ui::ClearAllBreakpoints,
                ),
            ],
        },
        Menu {
            name: t!("app_menus.window_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(t!("app_menus.window.minimize"), super::Minimize),
                MenuItem::action(t!("app_menus.window.zoom"), super::Zoom),
                MenuItem::separator(),
            ],
        },
        Menu {
            name: t!("app_menus.help_title").into(),
            disabled: false,
            items: vec![
                MenuItem::action(
                    t!("app_menus.help.release_notes"),
                    auto_update_ui::ViewReleaseNotesLocally,
                ),
                MenuItem::action(t!("app_menus.help.telemetry"), zed_actions::OpenTelemetryLog),
                MenuItem::action(t!("app_menus.help.licenses"), zed_actions::OpenLicenses),
                MenuItem::action(t!("app_menus.help.welcome"), onboarding::ShowWelcome),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.help.bug_report"),
                    zed_actions::feedback::FileBugReport,
                ),
                MenuItem::action(
                    t!("app_menus.help.request_feature"),
                    zed_actions::feedback::RequestFeature,
                ),
                MenuItem::action(t!("app_menus.help.email"), zed_actions::feedback::EmailZed),
                MenuItem::separator(),
                MenuItem::action(
                    t!("app_menus.help.documentation"),
                    super::OpenBrowser {
                        url: "https://zed.dev/docs".into(),
                    },
                ),
                MenuItem::action(t!("app_menus.help.repository"), feedback::OpenZedRepo),
                MenuItem::action(
                    t!("app_menus.help.twitter"),
                    super::OpenBrowser {
                        url: "https://twitter.com/zeddotdev".into(),
                    },
                ),
                MenuItem::action(
                    t!("app_menus.help.join_team"),
                    super::OpenBrowser {
                        url: "https://zed.dev/jobs".into(),
                    },
                ),
            ],
        },
    ]
}
