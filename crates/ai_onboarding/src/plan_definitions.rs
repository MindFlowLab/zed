use gpui::{IntoElement, ParentElement};
use ui::{List, ListBulletItem, prelude::*};
use zed_i18n::t;

/// Centralized definitions for Zed AI plans
pub struct PlanDefinitions;

impl PlanDefinitions {
    pub fn free_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.free_edit_predictions"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.free_unlimited_prompts_api_keys"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.free_unlimited_external_agents"
            )))
    }

    pub fn sign_in_upsell(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.tokens_20"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.no_credit_card"
            )))
    }

    pub fn pro_trial(&self, period: bool) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.tokens_20"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
            .when(period, |this| {
                this.child(ListBulletItem::new(t!(
                    "ai_onboarding.plan_definitions.trial_14_days"
                )))
            })
    }

    pub fn pro_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.tokens_5"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.usage_billing_beyond_5"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
    }

    pub fn business_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.usage_billing"
            )))
    }

    pub fn vip_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.tokens_in_agent"
            )))
    }

    pub fn student_plan(&self) -> impl IntoElement {
        List::new()
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.unlimited_edit_predictions"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.tokens_10"
            )))
            .child(ListBulletItem::new(t!(
                "ai_onboarding.plan_definitions.optional_credit_packs"
            )))
    }
}
