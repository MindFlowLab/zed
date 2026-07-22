use cloud_api_types::Plan;
use ui::{Chip, prelude::*};
use zed_i18n::t;

/// A [`Chip`] that displays a [`Plan`].
#[derive(IntoElement)]
pub struct PlanChip {
    plan: Plan,
}

impl PlanChip {
    pub fn new(plan: Plan) -> Self {
        Self { plan }
    }
}

impl RenderOnce for PlanChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let free_chip_bg = cx
            .theme()
            .colors()
            .editor_background
            .opacity(0.5)
            .blend(cx.theme().colors().text_accent.opacity(0.05));

        let pro_chip_bg = cx
            .theme()
            .colors()
            .editor_background
            .opacity(0.5)
            .blend(cx.theme().colors().text_accent.opacity(0.2));

        let (plan_name, label_color, bg_color) = match self.plan {
            Plan::ZedFree => (t!("title_bar.plan.free"), Color::Default, free_chip_bg),
            Plan::ZedProTrial => (t!("title_bar.plan.pro_trial"), Color::Accent, pro_chip_bg),
            Plan::ZedPro => (t!("title_bar.plan.pro"), Color::Accent, pro_chip_bg),
            Plan::ZedBusiness => (t!("title_bar.plan.business"), Color::Accent, pro_chip_bg),
            Plan::ZedVip => (t!("title_bar.plan.vip"), Color::Accent, pro_chip_bg),
            Plan::ZedStudent => (t!("title_bar.plan.student"), Color::Accent, pro_chip_bg),
        };

        Chip::new(plan_name)
            .bg_color(bg_color)
            .label_color(label_color)
    }
}
