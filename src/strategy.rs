//! Modular, state-aware action advice for the campaign simulator.
//!
//! Recommendation scores are decision heuristics over this game's modeled
//! support, context, and resources. They are not empirical vote estimates.

use crate::sim::{actions_for, ActionPreview, GameError, GameState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategyProfile {
    Balanced,
    Persuasion,
    IssueResponse,
    Mobilization,
    TrustBuilding,
    Sustainability,
}

impl StrategyProfile {
    pub const ALL: [Self; 6] = [
        Self::Balanced,
        Self::Persuasion,
        Self::IssueResponse,
        Self::Mobilization,
        Self::TrustBuilding,
        Self::Sustainability,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Persuasion => "persuasion",
            Self::IssueResponse => "issue",
            Self::Mobilization => "mobilization",
            Self::TrustBuilding => "trust",
            Self::Sustainability => "sustainable",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "均衡策略",
            Self::Persuasion => "支持拓展",
            Self::IssueResponse => "議題回應",
            Self::Mobilization => "投票動員",
            Self::TrustBuilding => "信任修復",
            Self::Sustainability => "資源續航",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "balanced" | "均衡" => Ok(Self::Balanced),
            "persuasion" | "support" | "支持" => Ok(Self::Persuasion),
            "issue" | "issues" | "議題" => Ok(Self::IssueResponse),
            "mobilization" | "turnout" | "動員" => Ok(Self::Mobilization),
            "trust" | "信任" => Ok(Self::TrustBuilding),
            "sustainable" | "resource" | "續航" => Ok(Self::Sustainability),
            _ => Err(format!(
                "策略請用 balanced、persuasion、issue、mobilization、trust 或 sustainable，收到：{value}"
            )),
        }
    }

    fn weight(self, axis: StrategyAxis) -> f64 {
        match (self, axis) {
            (Self::Balanced, _) => 1.0,
            (Self::Persuasion, StrategyAxis::Support) => 2.0,
            (Self::Persuasion, StrategyAxis::Context) => 0.7,
            (Self::Persuasion, StrategyAxis::Trust) => 0.5,
            (Self::Persuasion, StrategyAxis::Mobilization) => 0.4,
            (Self::Persuasion, StrategyAxis::Objective) => 0.5,
            (Self::Persuasion, StrategyAxis::Sustainability) => 0.5,
            (Self::IssueResponse, StrategyAxis::Support) => 0.8,
            (Self::IssueResponse, StrategyAxis::Context) => 2.0,
            (Self::IssueResponse, StrategyAxis::Trust) => 0.7,
            (Self::IssueResponse, StrategyAxis::Mobilization) => 0.6,
            (Self::IssueResponse, StrategyAxis::Objective) => 0.6,
            (Self::IssueResponse, StrategyAxis::Sustainability) => 0.5,
            (Self::Mobilization, StrategyAxis::Support) => 0.7,
            (Self::Mobilization, StrategyAxis::Context) => 0.6,
            (Self::Mobilization, StrategyAxis::Trust) => 0.5,
            (Self::Mobilization, StrategyAxis::Mobilization) => 2.0,
            (Self::Mobilization, StrategyAxis::Objective) => 0.6,
            (Self::Mobilization, StrategyAxis::Sustainability) => 0.4,
            (Self::TrustBuilding, StrategyAxis::Support) => 0.65,
            (Self::TrustBuilding, StrategyAxis::Context) => 0.7,
            (Self::TrustBuilding, StrategyAxis::Trust) => 2.0,
            (Self::TrustBuilding, StrategyAxis::Mobilization) => 0.6,
            (Self::TrustBuilding, StrategyAxis::Objective) => 0.6,
            (Self::TrustBuilding, StrategyAxis::Sustainability) => 0.55,
            (Self::Sustainability, StrategyAxis::Support) => 0.7,
            (Self::Sustainability, StrategyAxis::Context) => 0.5,
            (Self::Sustainability, StrategyAxis::Trust) => 0.8,
            (Self::Sustainability, StrategyAxis::Mobilization) => 0.75,
            (Self::Sustainability, StrategyAxis::Objective) => 0.75,
            (Self::Sustainability, StrategyAxis::Sustainability) => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategyAxis {
    Support,
    Context,
    Trust,
    Mobilization,
    Objective,
    Sustainability,
}

/// A scoring component that can be composed into a `StrategyEngine`.
pub trait StrategyModule {
    fn axis(&self) -> StrategyAxis;
    fn score(&self, game: &GameState, preview: &ActionPreview) -> f64;
    fn rationale(&self, game: &GameState, preview: &ActionPreview) -> Option<&'static str>;
}

#[derive(Clone, Debug)]
pub struct StrategyRecommendation {
    pub preview: ActionPreview,
    pub score: f64,
    pub rationales: Vec<&'static str>,
}

pub struct StrategyEngine {
    profile: StrategyProfile,
    modules: Vec<Box<dyn StrategyModule>>,
}

impl StrategyEngine {
    pub fn new(profile: StrategyProfile) -> Self {
        Self {
            profile,
            modules: vec![
                Box::new(SupportImpactModule),
                Box::new(ContextFitModule),
                Box::new(TrustModule),
                Box::new(MobilizationModule),
                Box::new(WeeklyObjectiveModule),
                Box::new(SustainabilityModule),
            ],
        }
    }

    pub fn with_module(mut self, module: Box<dyn StrategyModule>) -> Self {
        self.modules.push(module);
        self
    }

    pub fn profile(&self) -> StrategyProfile {
        self.profile
    }

    pub fn rank_actions(&self, game: &GameState) -> Vec<StrategyRecommendation> {
        let mut recommendations = actions_for(game.side)
            .iter()
            .filter_map(|action| {
                let preview = game.preview_action(action.id).ok()?;
                let mut score = 0.0;
                let mut rationales = Vec::new();
                for module in &self.modules {
                    score += module.score(game, &preview) * self.profile.weight(module.axis());
                    if let Some(rationale) = module.rationale(game, &preview) {
                        if !rationales.contains(&rationale) {
                            rationales.push(rationale);
                        }
                    }
                }
                Some(StrategyRecommendation {
                    preview,
                    score,
                    rationales,
                })
            })
            .collect::<Vec<_>>();
        recommendations.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| left.preview.action_name.cmp(right.preview.action_name))
        });
        recommendations
    }

    pub fn recommend(&self, game: &GameState) -> Result<StrategyRecommendation, GameError> {
        self.rank_actions(game)
            .into_iter()
            .next()
            .ok_or(GameError::ActionUnavailable)
    }
}

struct SupportImpactModule;

impl StrategyModule for SupportImpactModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Support
    }

    fn score(&self, _game: &GameState, preview: &ActionPreview) -> f64 {
        preview.expected_support_delta * 4.0
    }

    fn rationale(&self, _game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        if preview.expected_support_delta > 0.05 {
            Some("預估淨支持度上升")
        } else if preview.expected_support_delta < -0.05 {
            Some("含平均對手回應後支持度可能下修")
        } else {
            None
        }
    }
}

struct ContextFitModule;

impl StrategyModule for ContextFitModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Context
    }

    fn score(&self, _game: &GameState, preview: &ActionPreview) -> f64 {
        f64::from(preview.event_match) * 1.4 + f64::from(preview.local_match) * 1.0
    }

    fn rationale(&self, _game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        match (preview.event_match, preview.local_match) {
            (true, true) => Some("同時回應本週事件與地方焦點"),
            (true, false) => Some("回應本週事件焦點"),
            (false, true) => Some("符合地方議題焦點"),
            (false, false) => None,
        }
    }
}

struct TrustModule;

impl StrategyModule for TrustModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Trust
    }

    fn score(&self, game: &GameState, preview: &ActionPreview) -> f64 {
        let trust_need = 0.8 + (100.0 - game.trust) / 100.0;
        preview.trust_delta * trust_need
    }

    fn rationale(&self, game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        if game.trust < 45.0 && preview.trust_delta > 0.0 {
            Some("目前信任偏低，能修復信任資產")
        } else if preview.trust_delta > 0.2 {
            Some("有助累積信任資產")
        } else {
            None
        }
    }
}

struct MobilizationModule;

impl StrategyModule for MobilizationModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Mobilization
    }

    fn score(&self, game: &GameState, preview: &ActionPreview) -> f64 {
        let late_campaign_weight = if game.round >= 8 { 2.8 } else { 1.8 };
        preview.turnout_delta * late_campaign_weight
    }

    fn rationale(&self, game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        if game.round >= 8 && preview.turnout_delta > 0.0 {
            Some("決戰期可增加投票動能")
        } else if preview.turnout_delta >= 0.2 {
            Some("能累積投票動能")
        } else {
            None
        }
    }
}

struct WeeklyObjectiveModule;

impl StrategyModule for WeeklyObjectiveModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Objective
    }

    fn score(&self, game: &GameState, preview: &ActionPreview) -> f64 {
        if !preview.objective_match {
            return 0.0;
        }
        let completes_this_week = game.current_objective().is_some_and(|objective| {
            game.current_objective_progress().saturating_add(1) >= objective.target_actions
        });
        if completes_this_week {
            1.1
        } else {
            0.65
        }
    }

    fn rationale(&self, _game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        preview.objective_match.then_some("推進本週戰役任務")
    }
}

struct SustainabilityModule;

impl StrategyModule for SustainabilityModule {
    fn axis(&self) -> StrategyAxis {
        StrategyAxis::Sustainability
    }

    fn score(&self, game: &GameState, preview: &ActionPreview) -> f64 {
        let budget_pressure = 1.0 - (f64::from(game.budget) / 120.0).clamp(0.0, 1.0);
        let budget_cost = f64::from(preview.cost) * (0.08 + budget_pressure * 0.22);
        let stamina_pressure = if game.stamina < 45.0 { 0.035 } else { 0.012 };
        let stamina_change = if preview.stamina_delta >= 0.0 {
            preview.stamina_delta * 0.02
        } else {
            preview.stamina_delta * stamina_pressure
        };
        -budget_cost + stamina_change
    }

    fn rationale(&self, game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
        if game.budget <= 15 && preview.cost == 0 {
            Some("預算吃緊時仍可執行")
        } else if game.stamina < 25.0 && preview.stamina_delta > 0.0 {
            Some("體力偏低，能恢復續航")
        } else if preview.cost == 0 {
            Some("不消耗預算")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{CampaignStyle, Side};

    #[test]
    fn strategy_engine_ranks_legal_actions_without_mutating_game_state() {
        let game = GameState::new_with_style(2024, Side::Challenger, 55, CampaignStyle::Balanced);
        let before_polls = game.polls.clone();
        let before_budget = game.budget;
        let recommendations = StrategyEngine::new(StrategyProfile::Balanced).rank_actions(&game);

        assert!(!recommendations.is_empty());
        assert!(recommendations
            .windows(2)
            .all(|pair| pair[0].score >= pair[1].score));
        assert_eq!(game.polls, before_polls);
        assert_eq!(game.budget, before_budget);
    }

    #[test]
    fn strategy_engine_excludes_actions_the_campaign_cannot_afford() {
        let mut game =
            GameState::new_with_style(2024, Side::Challenger, 56, CampaignStyle::Balanced);
        game.budget = 0;
        let recommendations = StrategyEngine::new(StrategyProfile::Balanced).rank_actions(&game);

        assert!(!recommendations.is_empty());
        assert!(recommendations
            .iter()
            .all(|recommendation| { recommendation.preview.cost <= game.budget }));
    }

    #[test]
    fn strategy_profiles_are_named_and_parseable() {
        for profile in StrategyProfile::ALL {
            assert_eq!(StrategyProfile::parse(profile.key()).unwrap(), profile);
            assert!(!profile.label().is_empty());
        }
    }

    #[test]
    fn strategy_profiles_weight_the_same_action_differently() {
        let game = GameState::new_with_style(2024, Side::Challenger, 57, CampaignStyle::Balanced);
        let score = |profile| {
            StrategyEngine::new(profile)
                .rank_actions(&game)
                .into_iter()
                .find(|recommendation| recommendation.preview.action_name == "政見論壇與辯論")
                .unwrap()
                .score
        };

        assert_ne!(
            score(StrategyProfile::Persuasion),
            score(StrategyProfile::Sustainability)
        );
    }

    struct EventOnlyModule;

    impl StrategyModule for EventOnlyModule {
        fn axis(&self) -> StrategyAxis {
            StrategyAxis::Context
        }

        fn score(&self, _game: &GameState, preview: &ActionPreview) -> f64 {
            f64::from(preview.event_match)
        }

        fn rationale(&self, _game: &GameState, preview: &ActionPreview) -> Option<&'static str> {
            preview.event_match.then_some("自訂事件模組")
        }
    }

    #[test]
    fn strategy_modules_can_be_composed() {
        let game = GameState::new_with_style(2024, Side::Challenger, 2, CampaignStyle::Balanced);
        let engine =
            StrategyEngine::new(StrategyProfile::Balanced).with_module(Box::new(EventOnlyModule));
        let recommendations = engine.rank_actions(&game);

        assert!(recommendations
            .iter()
            .all(|recommendation| recommendation.score.is_finite()));
        assert!(recommendations
            .iter()
            .any(|recommendation| recommendation.rationales.contains(&"自訂事件模組")));
    }
}
