use ui::{
    ActiveTheme as _, AnyElement, ButtonSize, Checkbox, Context, ContextMenu, DropdownMenu,
    ElementId, IntoElement as _, ParentElement as _, Styled as _, ToggleState, Tooltip, Window,
    div, h_flex,
};
use zed_i18n::t;

use crate::{
    CsvPreviewView,
    settings::{FilterSortOrder, VerticalAlignment},
};

///// Settings related /////
impl CsvPreviewView {
    /// Render settings panel above the table
    pub(crate) fn render_settings_panel(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let current_alignment_text = match self.settings.vertical_alignment {
            VerticalAlignment::Top => t!("csv_preview.settings.alignment_top"),
            VerticalAlignment::Center => t!("csv_preview.settings.alignment_center"),
        };

        let current_filter_sort_text = match self.settings.filter_sort_order {
            FilterSortOrder::AlphaThenCount => t!("csv_preview.settings.filter_sort_alpha_then_count"),
            FilterSortOrder::CountThenAlpha => t!("csv_preview.settings.filter_sort_count_then_alpha"),
        };

        let view = cx.entity();
        let alignment_dropdown_menu = ContextMenu::build(window, cx, |menu, _window, _cx| {
            menu.entry(t!("csv_preview.settings.alignment_top"), None, {
                let view = view.clone();
                move |_window, cx| {
                    view.update(cx, |this, cx| {
                        this.settings.vertical_alignment = VerticalAlignment::Top;
                        cx.notify();
                    });
                }
            })
            .entry(t!("csv_preview.settings.alignment_center"), None, {
                let view = view.clone();
                move |_window, cx| {
                    view.update(cx, |this, cx| {
                        this.settings.vertical_alignment = VerticalAlignment::Center;
                        cx.notify();
                    });
                }
            })
        });

        let filter_sort_dropdown_menu = ContextMenu::build(window, cx, |menu, _window, _cx| {
            menu.entry(t!("csv_preview.settings.filter_sort_alpha_then_count"), None, {
                let view = view.clone();
                move |_window, cx| {
                    view.update(cx, |this, cx| {
                        this.settings.filter_sort_order = FilterSortOrder::AlphaThenCount;
                        cx.notify();
                    });
                }
            })
            .entry(t!("csv_preview.settings.filter_sort_count_then_alpha"), None, {
                let view = view.clone();
                move |_window, cx| {
                    view.update(cx, |this, cx| {
                        this.settings.filter_sort_order = FilterSortOrder::CountThenAlpha;
                        cx.notify();
                    });
                }
            })
        });

        let panel = h_flex()
            .gap_4()
            .p_2()
            .bg(cx.theme().colors().surface_background)
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .flex_wrap()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().colors().text_muted)
                            .child(t!("csv_preview.settings.text_alignment_label")),
                    )
                    .child(
                        DropdownMenu::new(
                            ElementId::Name("vertical-alignment-dropdown".into()),
                            current_alignment_text,
                            alignment_dropdown_menu,
                        )
                        .trigger_size(ButtonSize::Compact)
                        .trigger_tooltip(Tooltip::text(t!(
                            "csv_preview.settings.text_alignment_tooltip"
                        ))),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().colors().text_muted)
                            .child(t!("csv_preview.settings.filter_sort_label")),
                    )
                    .child(
                        DropdownMenu::new(
                            ElementId::Name("filter-sort-order-dropdown".into()),
                            current_filter_sort_text,
                            filter_sort_dropdown_menu,
                        )
                        .trigger_size(ButtonSize::Compact)
                        .trigger_tooltip(Tooltip::text(t!(
                            "csv_preview.settings.filter_sort_tooltip"
                        ))),
                    ),
            );

        let multiline_enabled = self.settings.multiline_cells_enabled;
        let panel = panel.child({
            let view = view.clone();
            Checkbox::new(
                ElementId::Name("multiline-rows-checkbox".into()),
                if multiline_enabled {
                    ToggleState::Selected
                } else {
                    ToggleState::Unselected
                },
            )
            .label(t!("csv_preview.settings.multiline_label"))
            .tooltip(Tooltip::text(t!("csv_preview.settings.multiline_tooltip")))
            .on_click(move |_state, _window, cx| {
                view.update(cx, |this, cx| {
                    this.settings.multiline_cells_enabled = !this.settings.multiline_cells_enabled;
                    cx.notify();
                });
            })
        });

        #[cfg(feature = "dev-tools")]
        let panel = panel.child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().colors().text_muted)
                        .child("Dev-only:"),
                )
                .child(create_dev_only_popover_menu(cx)),
        );

        panel.into_any_element()
    }
}

#[cfg(feature = "dev-tools")]
fn create_dev_only_popover_menu(
    cx: &mut Context<'_, CsvPreviewView>,
) -> ui::PopoverMenu<ContextMenu> {
    use crate::settings::RowRenderMechanism;
    use ui::{IconButton, IconName, IconPosition, IconSize, PopoverMenu};

    PopoverMenu::new("debug-options-menu")
        .trigger_with_tooltip(
            IconButton::new("debug-options-trigger", IconName::Settings).icon_size(IconSize::Small),
            Tooltip::text(
                "Dev-only section used for debugging purposes.\nWill be removed on public release of CSV feature"
            ),
        )
        .menu({
            let view_entity = cx.entity();
            move |window, cx| {
                let view = view_entity.read(cx);
                let settings = view.settings.clone();
                Some(ContextMenu::build(window, cx, |menu, _, _| {
                    menu.header("Rendering Mode")
                        .toggleable_entry(
                            "Variable Height",
                            settings.rendering_with == RowRenderMechanism::VariableList,
                            IconPosition::Start,
                            None,
                            {
                                let view_entity = view_entity.clone();
                                move |_w, cx| {
                                    view_entity.update(cx, |view, cx| {
                                        view.settings.rendering_with =
                                            RowRenderMechanism::VariableList;
                                        cx.notify();
                                    })
                                }
                            },
                        )
                        .toggleable_entry(
                            "Uniform Height",
                            settings.rendering_with == RowRenderMechanism::UniformList,
                            IconPosition::Start,
                            None,
                            {
                                let view_entity = view_entity.clone();
                                move |_w, cx| {
                                    view_entity.update(cx, |view, cx| {
                                        view.settings.rendering_with =
                                            RowRenderMechanism::UniformList;
                                        cx.notify();
                                    })
                                }
                            },
                        )
                        .separator()
                        .toggleable_entry(
                            "Show perf metrics",
                            settings.show_perf_metrics_overlay,
                            IconPosition::Start,
                            None,
                            {
                                let view_entity = view_entity.clone();
                                move |_w, cx| {
                                    view_entity.update(cx, |view, cx| {
                                        view.settings.show_perf_metrics_overlay =
                                            !view.settings.show_perf_metrics_overlay;
                                        cx.notify();
                                    })
                                }
                            },
                        )
                        .toggleable_entry(
                            "Show cell positions",
                            settings.show_debug_info,
                            IconPosition::Start,
                            None,
                            {
                                let view_entity = view_entity.clone();
                                move |_, cx| {
                                    view_entity.update(cx, |view, cx| {
                                        view.settings.show_debug_info =
                                            !view.settings.show_debug_info;
                                        cx.notify();
                                    })
                                }
                            },
                        )
                }))
            }
        })
}
