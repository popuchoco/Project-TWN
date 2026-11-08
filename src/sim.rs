use crate::history::{
    baseline_election, challenger_candidate_index, events_for_campaign, incumbent_candidate_index,
    party_affinity, EventCard, Issue, PartyProfile, PresidentialElection, RegionProfile,
    VOTER_SEGMENTS,
};
use std::fmt;

pub const CAMPAIGN_ROUNDS: u8 = 12;
pub const DAYS_PER_WEEK: u8 = 7;
const DEFAULT_SEED: u64 = 0x0054_574E_5F53_494D;
const MAX_BUDGET: i32 = 120;
const WEEKLY_FUNDING: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Incumbent,
    Challenger,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    CampaignComplete,
    AlreadySettled,
    ActionUnavailable,
    InsufficientBudget { required: i32, available: i32 },
    RequiresThreeCandidates,
    PollAlreadyTaken,
    CoalitionAlreadyUsed,
}

impl fmt::Display for GameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CampaignComplete => formatter.write_str("競選期已結束。"),
            Self::AlreadySettled => formatter.write_str("本場競選已結算。"),
            Self::ActionUnavailable => formatter.write_str("這項行動不適用於目前陣營。"),
            Self::InsufficientBudget {
                required,
                available,
            } => write!(formatter, "預算不足（需要 {required}，剩餘 {available}）。"),
            Self::RequiresThreeCandidates => {
                formatter.write_str("至少三位候選人時才可進行多候選人民調協調。")
            }
            Self::PollAlreadyTaken => formatter.write_str("本週已完成民調協調；每週最多一次。"),
            Self::CoalitionAlreadyUsed => formatter.write_str("本週已進行跨黨協商；每週最多一次。"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignPhase {
    Foundation,
    IssueCompetition,
    GetOutTheVote,
    Complete,
}

impl CampaignPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Foundation => "布局期",
            Self::IssueCompetition => "議題攻防期",
            Self::GetOutTheVote => "決戰期",
            Self::Complete => "競選期已結束",
        }
    }
}

impl Side {
    pub fn label(self) -> &'static str {
        match self {
            Side::Incumbent => "尋求連任／執政方",
            Side::Challenger => "在野挑戰方",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignStyle {
    Balanced,
    Policy,
    Organizer,
    Communicator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensitivityParameter {
    ActionSwing,
    EventBonus,
    LocalFocusBonus,
    TrustModifier,
    FatiguePenalty,
    DailyNoise,
    TerminalNoise,
}

impl SensitivityParameter {
    pub const ALL: [Self; 7] = [
        Self::ActionSwing,
        Self::EventBonus,
        Self::LocalFocusBonus,
        Self::TrustModifier,
        Self::FatiguePenalty,
        Self::DailyNoise,
        Self::TerminalNoise,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::ActionSwing => "action_swing",
            Self::EventBonus => "event_bonus",
            Self::LocalFocusBonus => "local_focus_bonus",
            Self::TrustModifier => "trust_modifier",
            Self::FatiguePenalty => "fatigue_penalty",
            Self::DailyNoise => "daily_noise",
            Self::TerminalNoise => "terminal_noise",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ActionSwing => "行動基礎效果",
            Self::EventBonus => "事件議題加成",
            Self::LocalFocusBonus => "地方焦點加成",
            Self::TrustModifier => "信任修正",
            Self::FatiguePenalty => "疲勞懲罰",
            Self::DailyNoise => "每日隨機波動",
            Self::TerminalNoise => "終局誤差範圍",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModelParameters {
    action_swing: f64,
    event_bonus: f64,
    local_focus_bonus: f64,
    trust_modifier: f64,
    fatigue_penalty: f64,
    daily_noise: f64,
    terminal_noise: f64,
}

impl ModelParameters {
    pub fn with_multiplier(
        mut self,
        parameter: SensitivityParameter,
        multiplier: f64,
    ) -> Result<Self, &'static str> {
        if !multiplier.is_finite() || !(0.0..=3.0).contains(&multiplier) {
            return Err("model parameter multipliers must be finite and within 0..=3");
        }
        match parameter {
            SensitivityParameter::ActionSwing => self.action_swing = multiplier,
            SensitivityParameter::EventBonus => self.event_bonus = multiplier,
            SensitivityParameter::LocalFocusBonus => self.local_focus_bonus = multiplier,
            SensitivityParameter::TrustModifier => self.trust_modifier = multiplier,
            SensitivityParameter::FatiguePenalty => self.fatigue_penalty = multiplier,
            SensitivityParameter::DailyNoise => self.daily_noise = multiplier,
            SensitivityParameter::TerminalNoise => self.terminal_noise = multiplier,
        }
        Ok(self)
    }

    fn multiplier(self, parameter: SensitivityParameter) -> f64 {
        match parameter {
            SensitivityParameter::ActionSwing => self.action_swing,
            SensitivityParameter::EventBonus => self.event_bonus,
            SensitivityParameter::LocalFocusBonus => self.local_focus_bonus,
            SensitivityParameter::TrustModifier => self.trust_modifier,
            SensitivityParameter::FatiguePenalty => self.fatigue_penalty,
            SensitivityParameter::DailyNoise => self.daily_noise,
            SensitivityParameter::TerminalNoise => self.terminal_noise,
        }
    }
}

impl Default for ModelParameters {
    fn default() -> Self {
        Self {
            action_swing: 1.0,
            event_bonus: 1.0,
            local_focus_bonus: 1.0,
            trust_modifier: 1.0,
            fatigue_penalty: 1.0,
            daily_noise: 1.0,
            terminal_noise: 1.0,
        }
    }
}

impl CampaignStyle {
    pub fn label(self) -> &'static str {
        match self {
            CampaignStyle::Balanced => "均衡型",
            CampaignStyle::Policy => "政策型",
            CampaignStyle::Organizer => "組織型",
            CampaignStyle::Communicator => "溝通型",
        }
    }

    fn attributes(self) -> CandidateAttributes {
        match self {
            CampaignStyle::Balanced => CandidateAttributes {
                policy: 50.0,
                organization: 50.0,
                communication: 50.0,
            },
            CampaignStyle::Policy => CandidateAttributes {
                policy: 62.0,
                organization: 48.0,
                communication: 50.0,
            },
            CampaignStyle::Organizer => CandidateAttributes {
                policy: 48.0,
                organization: 62.0,
                communication: 50.0,
            },
            CampaignStyle::Communicator => CandidateAttributes {
                policy: 50.0,
                organization: 48.0,
                communication: 62.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CandidateAttributes {
    pub policy: f64,
    pub organization: f64,
    pub communication: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionId {
    PolicyDelivery,
    CrisisResponse,
    TransparencyReport,
    PublicServices,
    CommunityTour,
    PolicyForum,
    LegalAssembly,
    CoalitionTalks,
    OversightPetition,
    PublicMessage,
    RestAndPrepare,
    PollCoordination,
}

#[derive(Clone, Copy, Debug)]
pub struct WeeklyObjective {
    pub title: &'static str,
    pub description: &'static str,
    pub qualifying_actions: &'static [ActionId],
    pub target_actions: u8,
}

const INCUMBENT_WEEKLY_OBJECTIVES: &[WeeklyObjective] = &[
    WeeklyObjective {
        title: "交付政策成果",
        description: "以具體施政、危機應變或民生服務完成兩項行動。",
        qualifying_actions: &[
            ActionId::PolicyDelivery,
            ActionId::CrisisResponse,
            ActionId::PublicServices,
        ],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "公開資訊與回應質疑",
        description: "透過施政檢討或公開政策問答完成兩項溝通行動。",
        qualifying_actions: &[ActionId::TransparencyReport, ActionId::PublicMessage],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "穩定地方服務",
        description: "以民生服務或施政方案回應在地生活議題，完成兩項行動。",
        qualifying_actions: &[ActionId::PublicServices, ActionId::PolicyDelivery],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "維持競選續航",
        description: "安排政策準備與休息，完成兩次休息與準備。",
        qualifying_actions: &[ActionId::RestAndPrepare],
        target_actions: 2,
    },
];

const CHALLENGER_WEEKLY_OBJECTIVES: &[WeeklyObjective] = &[
    WeeklyObjective {
        title: "走入社區、擴大參與",
        description: "透過社區走訪或依法集會完成兩項公民參與行動。",
        qualifying_actions: &[ActionId::CommunityTour, ActionId::LegalAssembly],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "提出可檢驗的替代方案",
        description: "以政見論壇或監督提案完成兩項政策論述行動。",
        qualifying_actions: &[ActionId::PolicyForum, ActionId::OversightPetition],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "建立可信的監督與說明",
        description: "用監督提案或公開政策問答完成兩項溝通行動。",
        qualifying_actions: &[ActionId::OversightPetition, ActionId::PublicMessage],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "維持競選續航",
        description: "安排政策準備與休息，完成兩次休息與準備。",
        qualifying_actions: &[ActionId::RestAndPrepare],
        target_actions: 2,
    },
];

#[derive(Clone, Copy, Debug)]
pub struct ActionSpec {
    pub id: ActionId,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub base_swing: f64,
    pub trust_delta: f64,
    pub turnout_delta: f64,
    pub issues: &'static [Issue],
    pub stamina_cost: f64,
    pub stamina_recovery: f64,
}

const INCUMBENT_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: ActionId::PolicyDelivery,
        name: "政策落實",
        description: "以可量化的政策進度與公共服務表現爭取支持。",
        cost: 3,
        base_swing: 0.38,
        trust_delta: 0.18,
        turnout_delta: 0.08,
        issues: &[Issue::Economy, Issue::Governance, Issue::Local],
        stamina_cost: 1.6,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CrisisResponse,
        name: "公共危機應變",
        description: "提出防災、衛生或民生危機應變計畫；成效與透明度影響信任。",
        cost: 3,
        base_swing: 0.36,
        trust_delta: 0.28,
        turnout_delta: 0.10,
        issues: &[Issue::PublicHealth, Issue::Governance],
        stamina_cost: 1.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::TransparencyReport,
        name: "公開施政與檢討",
        description: "公開進度、預算及尚未完成事項，降低資訊落差。",
        cost: 1,
        base_swing: 0.24,
        trust_delta: 0.30,
        turnout_delta: 0.08,
        issues: &[Issue::Trust, Issue::Media, Issue::Governance],
        stamina_cost: 0.7,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicServices,
        name: "地方民生服務",
        description: "強化公共運輸、照護、教育或社區服務交付。",
        cost: 2,
        base_swing: 0.31,
        trust_delta: 0.19,
        turnout_delta: 0.19,
        issues: &[Issue::Local, Issue::Education, Issue::Rights],
        stamina_cost: 1.2,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CoalitionTalks,
        name: "第三勢力協商與政見交換",
        description: "依政黨議題接近度尋找協作對象；談成時可整合部分支持。",
        cost: 2,
        base_swing: 0.12,
        trust_delta: 0.20,
        turnout_delta: 0.12,
        issues: &[Issue::Civic, Issue::Governance, Issue::CrossStrait],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicMessage,
        name: "公開政策問答",
        description: "以低成本發布可核實的政策說明與問答。",
        cost: 0,
        base_swing: 0.10,
        trust_delta: 0.08,
        turnout_delta: 0.06,
        issues: &[Issue::Trust, Issue::Media],
        stamina_cost: 0.3,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::RestAndPrepare,
        name: "休息與政見準備",
        description: "恢復體力，整理政策資料並為下一週做準備。",
        cost: 0,
        base_swing: 0.0,
        trust_delta: 0.0,
        turnout_delta: 0.0,
        issues: &[],
        stamina_cost: 0.0,
        stamina_recovery: 3.5,
    },
    ActionSpec {
        id: ActionId::PollCoordination,
        name: "對比／互比民調與整合協商",
        description: "檢視多候選人民調與對手互比，再嘗試整合一位落後候選人的支持。",
        cost: 2,
        base_swing: 0.14,
        trust_delta: 0.12,
        turnout_delta: 0.08,
        issues: &[Issue::Governance, Issue::Trust],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
];

const CHALLENGER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: ActionId::CommunityTour,
        name: "社區走訪與傾聽",
        description: "進行公開的地方走訪，把居民回饋納入政策提案。",
        cost: 2,
        base_swing: 0.30,
        trust_delta: 0.22,
        turnout_delta: 0.22,
        issues: &[Issue::Local, Issue::Rights, Issue::Education],
        stamina_cost: 1.3,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PolicyForum,
        name: "政見論壇與辯論",
        description: "以可檢驗的政策選項與對手比較，強化政策可信度。",
        cost: 2,
        base_swing: 0.34,
        trust_delta: 0.24,
        turnout_delta: 0.12,
        issues: &[
            Issue::Economy,
            Issue::CrossStrait,
            Issue::Governance,
            Issue::PublicHealth,
        ],
        stamina_cost: 1.1,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::LegalAssembly,
        name: "合法集會／公民行動",
        description: "舉辦和平、依法申請的集會，提升議題能見度與參與。",
        cost: 2,
        base_swing: 0.26,
        trust_delta: 0.12,
        turnout_delta: 0.32,
        issues: &[Issue::Civic, Issue::Media, Issue::Education, Issue::Rights],
        stamina_cost: 1.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CoalitionTalks,
        name: "跨黨協調與組織整合",
        description: "協調理念相近的團體與地方組織，擴大合作基礎。",
        cost: 2,
        base_swing: 0.12,
        trust_delta: 0.20,
        turnout_delta: 0.12,
        issues: &[Issue::Civic, Issue::Governance, Issue::CrossStrait],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::OversightPetition,
        name: "監督提案與公開連署",
        description: "透過可查證資料提出監督議題、具體替代方案與公開連署。",
        cost: 1,
        base_swing: 0.28,
        trust_delta: 0.22,
        turnout_delta: 0.12,
        issues: &[Issue::Trust, Issue::Media, Issue::Governance],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicMessage,
        name: "公開政策問答",
        description: "以低成本發布可核實的政策說明與問答。",
        cost: 0,
        base_swing: 0.10,
        trust_delta: 0.08,
        turnout_delta: 0.06,
        issues: &[Issue::Trust, Issue::Media],
        stamina_cost: 0.3,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::RestAndPrepare,
        name: "休息與政見準備",
        description: "恢復體力，整理政策資料並為下一週做準備。",
        cost: 0,
        base_swing: 0.0,
        trust_delta: 0.0,
        turnout_delta: 0.0,
        issues: &[],
        stamina_cost: 0.0,
        stamina_recovery: 3.5,
    },
    ActionSpec {
        id: ActionId::PollCoordination,
        name: "對比／互比民調與整合協商",
        description: "檢視多候選人民調與對手互比，再嘗試整合一位落後候選人的支持。",
        cost: 2,
        base_swing: 0.14,
        trust_delta: 0.12,
        turnout_delta: 0.08,
        issues: &[Issue::Governance, Issue::Trust],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
];

pub fn actions_for(side: Side) -> &'static [ActionSpec] {
    match side {
        Side::Incumbent => INCUMBENT_ACTIONS,
        Side::Challenger => CHALLENGER_ACTIONS,
    }
}

#[derive(Clone, Debug)]
pub struct SimCandidate {
    pub name: String,
    pub party: &'static str,
    pub historical_share: f64,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub campaign_year: u16,
    pub baseline_year: u16,
    pub region: Option<&'static RegionProfile>,
    pub side: Side,
    pub style: CampaignStyle,
    model_parameters: ModelParameters,
    pub player_attributes: CandidateAttributes,
    pub player_index: usize,
    pub candidates: Vec<SimCandidate>,
    pub polls: Vec<f64>,
    pub voter_segments: Vec<VoterSegmentPoll>,
    pub budget: i32,
    pub trust: f64,
    pub turnout_momentum: f64,
    pub stamina: f64,
    pub round: u8,
    pub day: u8,
    pub events: Vec<&'static EventCard>,
    /// One seeded issue focus per campaign week. `events` remains the full
    /// eligible event pool displayed to the player.
    event_schedule: Vec<&'static EventCard>,
    weekly_objective_schedule: Vec<&'static WeeklyObjective>,
    weekly_objective_progress: u8,
    weekly_objective_completed: bool,
    objectives_completed: u8,
    action_uses: [u8; 12],
    matched_events: u8,
    pub poll_taken_this_week: bool,
    pub coalition_this_week: bool,
    final_outcome: Option<ElectionOutcome>,
    rng: Rng,
}

#[derive(Clone, Debug)]
pub struct ActionOutcome {
    pub action_name: &'static str,
    pub event_title: Option<&'static str>,
    pub share_before: f64,
    pub share_after: f64,
    pub expected_support_delta: f64,
    pub trust_after: f64,
    pub turnout_after: f64,
    pub stamina_after: f64,
    pub cost: i32,
    pub weekly_funding: i32,
    pub budget_after: i32,
    pub coordination_note: Option<String>,
    pub objective_title: Option<&'static str>,
    pub objective_progress: u8,
    pub objective_target: u8,
    pub objective_completed: bool,
    pub objective_reward: i32,
    pub week: u8,
    pub day: u8,
}

/// A deterministic, one-step estimate of an action before seeded noise is drawn.
/// It is a forecast from this simulation model, not a poll or election prediction.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionPreview {
    pub id: ActionId,
    pub action_name: &'static str,
    pub expected_swing: f64,
    pub direct_support_delta: f64,
    pub coordination_support_delta: f64,
    pub expected_response_delta: f64,
    pub expected_support_delta: f64,
    pub event_match: bool,
    pub local_match: bool,
    pub objective_match: bool,
    pub cost: i32,
    pub trust_delta: f64,
    pub turnout_delta: f64,
    pub stamina_delta: f64,
}

#[derive(Clone, Debug)]
pub struct VoterSegmentPoll {
    pub name: &'static str,
    pub weight_pct: f64,
    pub polls: Vec<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct CampaignScorecard {
    pub policy: f64,
    pub trust: f64,
    pub mobilization: f64,
    pub event_response: f64,
    pub resource_management: f64,
    pub versatility: f64,
}

impl CampaignScorecard {
    pub fn axes(self) -> [(&'static str, f64); 6] {
        [
            ("政策準備", self.policy),
            ("信任經營", self.trust),
            ("組織動員", self.mobilization),
            ("議題回應", self.event_response),
            ("資源管理", self.resource_management),
            ("行動多樣", self.versatility),
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElectionOutcome {
    pub shares: Vec<f64>,
    pub winner_index: usize,
    pub player_win_probability: f64,
}

#[derive(Clone, Copy, Debug)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1_u64 << 53) as f64)
    }

    fn between(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }
}

impl GameState {
    pub fn new(campaign_year: u16, side: Side, seed: u64) -> Self {
        Self::new_with_style(campaign_year, side, seed, CampaignStyle::Balanced)
    }

    pub fn new_with_style(campaign_year: u16, side: Side, seed: u64, style: CampaignStyle) -> Self {
        Self::new_with_style_and_parameters(
            campaign_year,
            side,
            seed,
            style,
            ModelParameters::default(),
        )
    }

    pub fn new_with_style_and_parameters(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        model_parameters: ModelParameters,
    ) -> Self {
        let election = baseline_election(campaign_year);
        Self::from_election(campaign_year, side, election, seed, style, model_parameters)
    }

    pub fn new_local_with_style(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        region: &'static RegionProfile,
    ) -> Self {
        Self::new_local_with_party(campaign_year, side, seed, style, region, None)
    }

    pub fn new_local_with_party(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        region: &'static RegionProfile,
        selected_party: Option<&'static PartyProfile>,
    ) -> Self {
        Self::new_local_with_party_and_parameters(
            campaign_year,
            side,
            seed,
            style,
            region,
            selected_party,
            ModelParameters::default(),
        )
    }

    pub fn new_local_with_party_and_parameters(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        region: &'static RegionProfile,
        selected_party: Option<&'static PartyProfile>,
        model_parameters: ModelParameters,
    ) -> Self {
        let mut game = Self::from_election(
            campaign_year,
            side,
            baseline_election(2024),
            seed,
            style,
            model_parameters,
        );
        game.baseline_year = 2022;
        game.region = Some(region);
        let (mut parties, shares) = local_slate(region);
        if let Some(selected_party) = selected_party {
            if !parties.contains(&selected_party.name) {
                parties[2] = selected_party.name;
            }
        }
        game.candidates = parties
            .iter()
            .zip(shares)
            .map(|(party, share)| SimCandidate {
                name: format!("{party}（地方情境代表）"),
                party,
                historical_share: share,
            })
            .collect();
        game.polls = shares.to_vec();
        game.voter_segments = make_voter_segments(&game.polls);
        game.player_index = match side {
            Side::Incumbent => 0,
            Side::Challenger => selected_party
                .and_then(|party| parties.iter().position(|name| *name == party.name))
                .unwrap_or(1),
        };
        game
    }

    fn from_election(
        campaign_year: u16,
        side: Side,
        election: &'static PresidentialElection,
        seed: u64,
        style: CampaignStyle,
        model_parameters: ModelParameters,
    ) -> Self {
        let player_index = match side {
            Side::Incumbent => incumbent_candidate_index(election),
            Side::Challenger => challenger_candidate_index(election),
        };
        let uses_historical_names = campaign_year == election.year;
        let candidates: Vec<SimCandidate> = election
            .candidates
            .iter()
            .map(|result| SimCandidate {
                name: if uses_historical_names {
                    format!("{} / {}", result.president, result.vice_president)
                } else {
                    format!("{}（情境代表）", result.party)
                },
                party: result.party,
                historical_share: result.share_pct,
            })
            .collect();
        let mut polls: Vec<f64> = candidates
            .iter()
            .map(|candidate| candidate.historical_share)
            .collect();
        normalize(&mut polls);
        let (trust, turnout_momentum) = match side {
            Side::Incumbent => (54.0, 50.0),
            Side::Challenger => (50.0, 50.0),
        };
        let voter_segments = make_voter_segments(&polls);
        let events = events_for_campaign(campaign_year);
        let mut rng = Rng::new(seed);
        let event_schedule = make_event_schedule(&events, &mut rng);
        let weekly_objective_schedule = make_weekly_objective_schedule(side, &mut rng);

        Self {
            campaign_year,
            baseline_year: election.year,
            region: None,
            side,
            style,
            model_parameters,
            player_attributes: style.attributes(),
            player_index,
            candidates,
            polls,
            voter_segments,
            budget: 100,
            trust,
            turnout_momentum,
            stamina: 100.0,
            round: 0,
            day: 0,
            events,
            event_schedule,
            weekly_objective_schedule,
            weekly_objective_progress: 0,
            weekly_objective_completed: false,
            objectives_completed: 0,
            action_uses: [0; 12],
            matched_events: 0,
            poll_taken_this_week: false,
            coalition_this_week: false,
            final_outcome: None,
            rng,
        }
    }

    pub fn current_event(&self) -> Option<&'static EventCard> {
        self.event_schedule.get(self.round as usize).copied()
    }

    pub fn current_objective(&self) -> Option<&'static WeeklyObjective> {
        self.weekly_objective_schedule
            .get(self.round as usize)
            .copied()
    }

    pub fn current_objective_progress(&self) -> u8 {
        self.weekly_objective_progress
    }

    fn action_spec(&self, id: ActionId) -> Result<ActionSpec, GameError> {
        if self.final_outcome.is_some() {
            return Err(GameError::AlreadySettled);
        }
        if self.round >= CAMPAIGN_ROUNDS {
            return Err(GameError::CampaignComplete);
        }
        let action = actions_for(self.side)
            .iter()
            .find(|action| action.id == id)
            .copied()
            .ok_or(GameError::ActionUnavailable)?;
        if self.budget < action.cost {
            return Err(GameError::InsufficientBudget {
                required: action.cost,
                available: self.budget,
            });
        }
        if id == ActionId::PollCoordination && self.candidates.len() < 3 {
            return Err(GameError::RequiresThreeCandidates);
        }
        if id == ActionId::PollCoordination && self.poll_taken_this_week {
            return Err(GameError::PollAlreadyTaken);
        }
        if id == ActionId::CoalitionTalks && self.candidates.len() < 3 {
            return Err(GameError::RequiresThreeCandidates);
        }
        if id == ActionId::CoalitionTalks && self.coalition_this_week {
            return Err(GameError::CoalitionAlreadyUsed);
        }
        Ok(action)
    }

    /// Estimate an action's immediate effect without consuming campaign state or RNG.
    pub fn preview_action(&self, id: ActionId) -> Result<ActionPreview, GameError> {
        let action = self.action_spec(id)?;
        Ok(self.preview_action_for(action))
    }

    fn preview_action_for(&self, action: ActionSpec) -> ActionPreview {
        let before = self.polls[self.player_index];
        let event_match = self
            .current_event()
            .is_some_and(|event| action.issues.contains(&event.issue));
        let local_match = self
            .region
            .is_some_and(|region| action.issues.contains(&region.focus));
        let objective_match = !self.weekly_objective_completed
            && self.current_objective().is_some_and(|objective| {
                objective.qualifying_actions.contains(&action.id)
                    && self.weekly_objective_progress < objective.target_actions
            });
        let expected_swing = self.expected_action_swing(action).clamp(-0.2, 0.55);
        let mut projected_polls = self.polls.clone();
        let mut projected_segments = self.voter_segments.clone();
        if expected_swing.abs() > f64::EPSILON {
            apply_segment_swing_to_state(
                &mut projected_polls,
                &mut projected_segments,
                self.player_index,
                expected_swing,
                action.issues,
            );
        }
        let direct_support_delta = projected_polls[self.player_index] - before;

        if action.id == ActionId::PollCoordination {
            if let Some(target) = weakest_rival_index(&projected_polls, self.player_index) {
                let success_chance = (0.30
                    + (self.trust - 50.0) * 0.003
                    + (self.player_attributes.communication - 50.0) * 0.006)
                    .clamp(0.12, 0.72);
                let expected_transfer_rate = success_chance * 0.06;
                transfer_segment_support_at_rate(
                    &mut projected_polls,
                    &mut projected_segments,
                    target,
                    self.player_index,
                    expected_transfer_rate,
                );
            }
        } else if action.id == ActionId::CoalitionTalks {
            if let Some(target) = self.most_compatible_rival() {
                let affinity = party_affinity(
                    self.candidates[self.player_index].party,
                    self.candidates[target].party,
                );
                let success_chance =
                    (0.14 + affinity * 0.005 + (self.trust - 50.0) * 0.002).clamp(0.08, 0.72);
                let expected_transfer_rate = success_chance * 0.065;
                transfer_segment_support_at_rate(
                    &mut projected_polls,
                    &mut projected_segments,
                    target,
                    self.player_index,
                    expected_transfer_rate,
                );
            }
        }
        let after_coordination = projected_polls[self.player_index];
        let coordination_support_delta = after_coordination - (before + direct_support_delta);

        if action.id != ActionId::RestAndPrepare {
            if let Some(rival_index) = strongest_rival_index(&projected_polls, self.player_index) {
                apply_segment_swing_to_state(
                    &mut projected_polls,
                    &mut projected_segments,
                    rival_index,
                    0.055,
                    &[],
                );
            }
        }
        let expected_support_after = projected_polls[self.player_index];
        let expected_response_delta = expected_support_after - after_coordination;
        let trust_after = (self.trust + action.trust_delta).clamp(0.0, 100.0);
        let turnout_after = (self.turnout_momentum + action.turnout_delta).clamp(0.0, 100.0);
        let stamina_after =
            (self.stamina - action.stamina_cost + action.stamina_recovery).clamp(0.0, 100.0);

        ActionPreview {
            id: action.id,
            action_name: action.name,
            expected_swing,
            direct_support_delta,
            coordination_support_delta,
            expected_response_delta,
            expected_support_delta: expected_support_after - before,
            event_match,
            local_match,
            objective_match,
            cost: action.cost,
            trust_delta: trust_after - self.trust,
            turnout_delta: turnout_after - self.turnout_momentum,
            stamina_delta: stamina_after - self.stamina,
        }
    }

    fn expected_action_swing(&self, action: ActionSpec) -> f64 {
        if action.id == ActionId::RestAndPrepare {
            return 0.0;
        }
        let issue_bonus = if self
            .current_event()
            .is_some_and(|event| action.issues.contains(&event.issue))
        {
            0.12 * self
                .model_parameters
                .multiplier(SensitivityParameter::EventBonus)
        } else {
            0.0
        };
        let local_match = if self
            .region
            .is_some_and(|region| action.issues.contains(&region.focus))
        {
            0.08 * self
                .model_parameters
                .multiplier(SensitivityParameter::LocalFocusBonus)
        } else {
            0.0
        };
        let trust_bonus = ((self.trust - 50.0)
            * 0.004
            * self
                .model_parameters
                .multiplier(SensitivityParameter::TrustModifier))
        .clamp(-0.2, 0.2);
        let skill = action_skill(self.player_attributes, action.id);
        let skill_bonus = ((skill - 50.0) * 0.006).clamp(-0.2, 0.2);
        let base_fatigue_penalty = if self.stamina < 20.0 {
            -0.12
        } else if self.stamina < 45.0 {
            -0.06
        } else {
            0.0
        };
        let fatigue_penalty = base_fatigue_penalty
            * self
                .model_parameters
                .multiplier(SensitivityParameter::FatiguePenalty);
        action.base_swing
            * self
                .model_parameters
                .multiplier(SensitivityParameter::ActionSwing)
            + issue_bonus
            + local_match
            + trust_bonus
            + skill_bonus
            + fatigue_penalty
    }

    pub fn objectives_completed(&self) -> u8 {
        self.objectives_completed
    }

    pub fn apply_action(&mut self, id: ActionId) -> Result<ActionOutcome, GameError> {
        let action = self.action_spec(id)?;
        let preview = self.preview_action_for(action);

        let share_before = self.polls[self.player_index];
        let week = self.round + 1;
        let day = self.day + 1;
        let current_objective = self.current_objective();
        let objective_title = current_objective.map(|objective| objective.title);
        let objective_target = current_objective
            .map(|objective| objective.target_actions)
            .unwrap_or(0);
        let current_event = self.current_event();
        let event_title = current_event.map(|event| event.title);
        let event_match = preview.event_match;
        let daily_noise = 0.07
            * self
                .model_parameters
                .multiplier(SensitivityParameter::DailyNoise);
        let jitter = self.rng.between(-daily_noise, daily_noise);
        let swing = if id == ActionId::RestAndPrepare {
            0.0
        } else {
            (self.expected_action_swing(action) + jitter).clamp(-0.2, 0.55)
        };

        self.budget -= action.cost;
        if swing.abs() > f64::EPSILON {
            self.apply_segment_swing(self.player_index, swing, action.issues);
        }

        let mut coordination_note = None;
        if id == ActionId::PollCoordination {
            if let Some(target) = self.weakest_rival() {
                let success_chance = (0.30
                    + (self.trust - 50.0) * 0.003
                    + (self.player_attributes.communication - 50.0) * 0.006)
                    .clamp(0.12, 0.72);
                if self.rng.unit() < success_chance {
                    let rate = self.rng.between(0.04, 0.08);
                    let transfer = self.transfer_segment_support(target, rate);
                    coordination_note = Some(format!(
                        "互比後協調成功：{} 的部分支持轉向玩家陣營（模型轉移 {:.1} 個百分點）",
                        self.candidates[target].party, transfer
                    ));
                } else {
                    coordination_note = Some(format!(
                        "互比結果已完成，但與 {} 的整合協商未達共識",
                        self.candidates[target].party
                    ));
                }
            }
            self.poll_taken_this_week = true;
        }
        if id == ActionId::CoalitionTalks {
            if let Some(target) = self.most_compatible_rival() {
                let affinity = party_affinity(
                    self.candidates[self.player_index].party,
                    self.candidates[target].party,
                );
                let success_chance =
                    (0.14 + affinity * 0.005 + (self.trust - 50.0) * 0.002).clamp(0.08, 0.72);
                if self.rng.unit() < success_chance {
                    let rate = self.rng.between(0.04, 0.09);
                    let transfer = self.transfer_segment_support(target, rate);
                    coordination_note = Some(format!(
                        "與 {} 協商成功（議題接近度 {:.0}/100），模型整合 {:.1} 個百分點支持；此為情境效果，非固定聯盟。",
                        self.candidates[target].party, affinity, transfer
                    ));
                } else {
                    coordination_note = Some(format!(
                        "已與 {} 接觸（議題接近度 {:.0}/100），但本週未形成合作共識。",
                        self.candidates[target].party, affinity
                    ));
                }
            }
            self.coalition_this_week = true;
        }

        // A competing campaign responds to daily activity with a small seeded
        // movement; this is a model parameter, not a historical vote estimate.
        if id != ActionId::RestAndPrepare {
            if let Some(rival_index) = self.strongest_rival() {
                let counter_swing = self.rng.between(0.02, 0.09);
                self.apply_segment_swing(rival_index, counter_swing, &[]);
            }
        }

        self.trust = (self.trust + action.trust_delta).clamp(0.0, 100.0);
        self.turnout_momentum = (self.turnout_momentum + action.turnout_delta).clamp(0.0, 100.0);
        self.stamina =
            (self.stamina - action.stamina_cost + action.stamina_recovery).clamp(0.0, 100.0);
        self.action_uses[action_index(id)] = self.action_uses[action_index(id)].saturating_add(1);
        let mut objective_completed = false;
        let mut objective_reward = 0;
        if let Some(objective) = current_objective {
            if objective.qualifying_actions.contains(&id) {
                self.weekly_objective_progress = self
                    .weekly_objective_progress
                    .saturating_add(1)
                    .min(objective.target_actions);
            }
            if !self.weekly_objective_completed
                && self.weekly_objective_progress >= objective.target_actions
            {
                self.weekly_objective_completed = true;
                self.objectives_completed = self.objectives_completed.saturating_add(1);
                let before_reward = self.budget;
                self.budget = (self.budget + 1).min(MAX_BUDGET);
                objective_reward = self.budget - before_reward;
                objective_completed = true;
            }
        }
        if event_match {
            self.matched_events = self.matched_events.saturating_add(1);
        }
        let objective_progress = self.weekly_objective_progress;
        let mut weekly_funding = 0;
        if day == DAYS_PER_WEEK {
            let before_funding = self.budget;
            self.budget = (self.budget + WEEKLY_FUNDING).min(MAX_BUDGET);
            weekly_funding = self.budget - before_funding;
            self.round += 1;
            self.day = 0;
            self.poll_taken_this_week = false;
            self.coalition_this_week = false;
            self.weekly_objective_progress = 0;
            self.weekly_objective_completed = false;
        } else {
            self.day += 1;
        }

        Ok(ActionOutcome {
            action_name: action.name,
            event_title,
            share_before,
            share_after: self.polls[self.player_index],
            expected_support_delta: preview.expected_support_delta,
            trust_after: self.trust,
            turnout_after: self.turnout_momentum,
            stamina_after: self.stamina,
            cost: action.cost,
            weekly_funding,
            budget_after: self.budget,
            coordination_note,
            objective_title,
            objective_progress,
            objective_target,
            objective_completed,
            objective_reward,
            week,
            day,
        })
    }

    pub fn phase_label(&self) -> &'static str {
        self.phase().label()
    }

    pub fn phase(&self) -> CampaignPhase {
        if self.is_finished() || self.is_settled() {
            CampaignPhase::Complete
        } else {
            match self.round / 4 {
                0 => CampaignPhase::Foundation,
                1 => CampaignPhase::IssueCompetition,
                _ => CampaignPhase::GetOutTheVote,
            }
        }
    }

    pub fn scorecard(&self) -> CampaignScorecard {
        let policy_actions = [
            ActionId::PolicyDelivery,
            ActionId::CrisisResponse,
            ActionId::PolicyForum,
        ]
        .iter()
        .map(|id| self.action_uses[action_index(*id)] as f64)
        .sum::<f64>();
        let unique_actions = self.action_uses.iter().filter(|count| **count > 0).count() as f64;
        let completed_days = (self.round as f64 * DAYS_PER_WEEK as f64) + self.day as f64;
        let policy_ratio = if completed_days > 0.0 {
            policy_actions / completed_days
        } else {
            0.0
        };
        CampaignScorecard {
            policy: (25.0 + policy_ratio * 500.0).clamp(0.0, 100.0),
            trust: self.trust,
            mobilization: self.turnout_momentum,
            event_response: (30.0 + (self.matched_events as f64 / completed_days.max(1.0)) * 180.0)
                .clamp(0.0, 100.0),
            resource_management: (55.0 + (self.budget as f64 * 0.25)).clamp(0.0, 100.0),
            versatility: (unique_actions * (100.0 / 8.0)).clamp(0.0, 100.0),
        }
    }

    pub fn finish(&mut self) -> ElectionOutcome {
        if let Some(outcome) = &self.final_outcome {
            return outcome.clone();
        }
        let mut player_wins = 0_usize;
        let mut final_shares = self.polls.clone();
        let samples = 1_200;
        // Mobilization changes the player's likely vote yield slightly rather
        // than merely appearing as a decorative meter (bounded at +/- 2 pts).
        let mobilization_effect = ((self.turnout_momentum - 50.0) * 0.04).clamp(-2.0, 2.0);
        let terminal_noise = 4.0
            * self
                .model_parameters
                .multiplier(SensitivityParameter::TerminalNoise);
        for sample in 0..samples {
            let mut ballot = self.polls.clone();
            let late_swing =
                self.rng.between(-terminal_noise, terminal_noise) + mobilization_effect;
            shift_share(&mut ballot, self.player_index, late_swing);
            if sample == 0 {
                final_shares = ballot.clone();
            }
            if winner_index(&ballot) == self.player_index {
                player_wins += 1;
            }
        }

        let outcome = ElectionOutcome {
            winner_index: winner_index(&final_shares),
            shares: final_shares,
            player_win_probability: player_wins as f64 / samples as f64,
        };
        self.final_outcome = Some(outcome.clone());
        outcome
    }

    pub fn is_finished(&self) -> bool {
        self.round >= CAMPAIGN_ROUNDS
    }

    pub fn is_settled(&self) -> bool {
        self.final_outcome.is_some()
    }

    /// Check the public state invariants at a test or future persistence boundary.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.candidates.len() < 2 || self.polls.len() != self.candidates.len() {
            return Err(
                "candidate and poll dimensions must match and contain at least two entries",
            );
        }
        if !is_valid_share_vector(&self.polls) {
            return Err("aggregate poll shares must be finite, within 0..=100, and sum to 100");
        }
        if self.voter_segments.is_empty() {
            return Err("at least one voter segment is required");
        }
        let segment_weight: f64 = self
            .voter_segments
            .iter()
            .map(|segment| segment.weight_pct)
            .sum();
        if !segment_weight.is_finite() || (segment_weight - 100.0).abs() > 0.0001 {
            return Err("voter segment weights must sum to 100");
        }
        for segment in &self.voter_segments {
            if !is_valid_share_vector(&segment.polls)
                || segment.polls.len() != self.candidates.len()
            {
                return Err("each voter segment must have a valid poll vector per candidate");
            }
        }
        for (index, aggregate) in self.polls.iter().enumerate() {
            let from_segments: f64 = self
                .voter_segments
                .iter()
                .map(|segment| segment.polls[index] * segment.weight_pct / 100.0)
                .sum();
            if (aggregate - from_segments).abs() > 0.0001 {
                return Err("aggregate polls must equal the weighted voter segment polls");
            }
        }
        if !(0..=MAX_BUDGET).contains(&self.budget) {
            return Err("budget must remain within the configured bounds");
        }
        if ![self.trust, self.turnout_momentum, self.stamina]
            .iter()
            .all(|value| value.is_finite() && (0.0..=100.0).contains(value))
        {
            return Err("trust, turnout, and stamina must remain within 0..=100");
        }
        if self.round > CAMPAIGN_ROUNDS || self.day >= DAYS_PER_WEEK {
            return Err("campaign clock is outside the configured schedule");
        }
        if self.round == CAMPAIGN_ROUNDS && self.day != 0 {
            return Err("a completed campaign must be at the first day boundary");
        }
        if !self.events.is_empty() && self.event_schedule.len() != CAMPAIGN_ROUNDS as usize {
            return Err("an eligible event pool must produce one scheduled focus per week");
        }
        if self.events.is_empty() && !self.event_schedule.is_empty() {
            return Err("event schedule must be empty when no events are eligible");
        }
        if self.weekly_objective_schedule.len() != CAMPAIGN_ROUNDS as usize
            || self.weekly_objective_schedule.iter().any(|objective| {
                objective.target_actions == 0
                    || objective.qualifying_actions.is_empty()
                    || objective
                        .qualifying_actions
                        .iter()
                        .any(|id| !actions_for(self.side).iter().any(|action| action.id == *id))
            })
        {
            return Err("each campaign week must have a valid objective");
        }
        if self.objectives_completed > self.round.saturating_add(1).min(CAMPAIGN_ROUNDS) {
            return Err("completed weekly objectives cannot exceed campaign weeks reached");
        }
        if let Some(objective) = self.current_objective() {
            if self.weekly_objective_progress > objective.target_actions {
                return Err("weekly objective progress cannot exceed its target");
            }
            if self.weekly_objective_completed
                && self.weekly_objective_progress < objective.target_actions
            {
                return Err("completed weekly objectives must have met their target");
            }
        } else if self.weekly_objective_progress != 0 || self.weekly_objective_completed {
            return Err("completed campaigns cannot retain in-progress weekly objectives");
        }
        if let Some(outcome) = &self.final_outcome {
            if outcome.winner_index >= self.candidates.len()
                || !is_valid_share_vector(&outcome.shares)
                || outcome.shares.len() != self.candidates.len()
                || !outcome.player_win_probability.is_finite()
                || !(0.0..=1.0).contains(&outcome.player_win_probability)
            {
                return Err(
                    "settled outcome must contain a valid winner, shares, and win probability",
                );
            }
        }
        Ok(())
    }

    fn strongest_rival(&self) -> Option<usize> {
        strongest_rival_index(&self.polls, self.player_index)
    }

    fn weakest_rival(&self) -> Option<usize> {
        weakest_rival_index(&self.polls, self.player_index)
    }

    fn most_compatible_rival(&self) -> Option<usize> {
        self.candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.player_index)
            .max_by(|(_, left), (_, right)| {
                party_affinity(self.candidates[self.player_index].party, left.party).total_cmp(
                    &party_affinity(self.candidates[self.player_index].party, right.party),
                )
            })
            .map(|(index, _)| index)
    }

    fn apply_segment_swing(&mut self, candidate_index: usize, swing: f64, issues: &[Issue]) {
        apply_segment_swing_to_state(
            &mut self.polls,
            &mut self.voter_segments,
            candidate_index,
            swing,
            issues,
        );
    }

    fn transfer_segment_support(&mut self, from: usize, rate: f64) -> f64 {
        let mut total_transfer = 0.0;
        for segment in &mut self.voter_segments {
            if from >= segment.polls.len() || self.player_index >= segment.polls.len() {
                continue;
            }
            let amount = segment.polls[from] * rate;
            transfer_share(&mut segment.polls, from, self.player_index, amount);
            total_transfer += amount * segment.weight_pct / 100.0;
        }
        self.sync_polls_from_segments();
        total_transfer
    }

    fn sync_polls_from_segments(&mut self) {
        if self.voter_segments.is_empty() {
            return;
        }
        self.polls.fill(0.0);
        for segment in &self.voter_segments {
            for (total, share) in self.polls.iter_mut().zip(&segment.polls) {
                *total += share * segment.weight_pct / 100.0;
            }
        }
        normalize(&mut self.polls);
    }
}

fn apply_segment_swing_to_state(
    polls: &mut [f64],
    voter_segments: &mut [VoterSegmentPoll],
    candidate_index: usize,
    swing: f64,
    issues: &[Issue],
) {
    if voter_segments.is_empty() {
        shift_share(polls, candidate_index, swing);
        return;
    }
    let factors: Vec<f64> = VOTER_SEGMENTS
        .iter()
        .map(|segment| {
            segment.responsiveness
                * if issues.contains(&segment.priority) {
                    1.45
                } else {
                    0.72
                }
        })
        .collect();
    let weighted_mean = VOTER_SEGMENTS
        .iter()
        .zip(&factors)
        .map(|(segment, factor)| segment.weight_pct * factor)
        .sum::<f64>()
        / 100.0;
    for (segment, factor) in voter_segments.iter_mut().zip(factors) {
        let segment_swing = if weighted_mean > f64::EPSILON {
            swing * factor / weighted_mean
        } else {
            swing
        };
        shift_share(&mut segment.polls, candidate_index, segment_swing);
    }
    sync_polls_from_segments(polls, voter_segments);
}

fn sync_polls_from_segments(polls: &mut [f64], voter_segments: &[VoterSegmentPoll]) {
    polls.fill(0.0);
    for segment in voter_segments {
        for (total, share) in polls.iter_mut().zip(&segment.polls) {
            *total += share * segment.weight_pct / 100.0;
        }
    }
    normalize(polls);
}

fn transfer_segment_support_at_rate(
    polls: &mut [f64],
    voter_segments: &mut [VoterSegmentPoll],
    from: usize,
    to: usize,
    rate: f64,
) {
    if voter_segments.is_empty() {
        let amount = polls.get(from).copied().unwrap_or(0.0) * rate;
        transfer_share(polls, from, to, amount);
        return;
    }
    for segment in voter_segments.iter_mut() {
        let amount = segment.polls.get(from).copied().unwrap_or(0.0) * rate;
        transfer_share(&mut segment.polls, from, to, amount);
    }
    sync_polls_from_segments(polls, voter_segments);
}

fn strongest_rival_index(polls: &[f64], player_index: usize) -> Option<usize> {
    polls
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != player_index)
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

fn weakest_rival_index(polls: &[f64], player_index: usize) -> Option<usize> {
    polls
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != player_index)
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
}

fn action_skill(attributes: CandidateAttributes, id: ActionId) -> f64 {
    match id {
        ActionId::PolicyDelivery | ActionId::CrisisResponse | ActionId::PolicyForum => {
            attributes.policy
        }
        ActionId::CommunityTour
        | ActionId::PublicServices
        | ActionId::LegalAssembly
        | ActionId::CoalitionTalks => attributes.organization,
        ActionId::TransparencyReport
        | ActionId::OversightPetition
        | ActionId::PublicMessage
        | ActionId::RestAndPrepare
        | ActionId::PollCoordination => attributes.communication,
    }
}

fn make_event_schedule(events: &[&'static EventCard], rng: &mut Rng) -> Vec<&'static EventCard> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut schedule = Vec::with_capacity(CAMPAIGN_ROUNDS as usize);
    let mut deck = Vec::new();
    while schedule.len() < CAMPAIGN_ROUNDS as usize {
        if deck.is_empty() {
            deck.extend_from_slice(events);
            for index in (1..deck.len()).rev() {
                let other = (rng.next_u64() % (index as u64 + 1)) as usize;
                deck.swap(index, other);
            }
            if deck.len() > 1
                && schedule
                    .last()
                    .is_some_and(|previous: &&'static EventCard| {
                        previous.title == deck.last().unwrap().title
                    })
            {
                let last = deck.len() - 1;
                deck.swap(last, last - 1);
            }
        }
        schedule.push(deck.pop().expect("event deck is refilled before use"));
    }
    schedule
}

fn make_weekly_objective_schedule(side: Side, rng: &mut Rng) -> Vec<&'static WeeklyObjective> {
    let objectives = match side {
        Side::Incumbent => INCUMBENT_WEEKLY_OBJECTIVES,
        Side::Challenger => CHALLENGER_WEEKLY_OBJECTIVES,
    };
    let mut schedule = Vec::with_capacity(CAMPAIGN_ROUNDS as usize);
    let mut deck: Vec<&'static WeeklyObjective> = Vec::new();

    while schedule.len() < CAMPAIGN_ROUNDS as usize {
        if deck.is_empty() {
            deck.extend(objectives);
            for index in (1..deck.len()).rev() {
                let other = (rng.next_u64() % (index as u64 + 1)) as usize;
                deck.swap(index, other);
            }
            if deck.len() > 1
                && schedule
                    .last()
                    .is_some_and(|previous: &&'static WeeklyObjective| {
                        previous.title == deck.last().expect("objective deck is not empty").title
                    })
            {
                let last = deck.len() - 1;
                deck.swap(last, last - 1);
            }
        }
        schedule.push(deck.pop().expect("objective deck is refilled before use"));
    }
    schedule
}

fn winner_index(shares: &[f64]) -> usize {
    shares
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn is_valid_share_vector(shares: &[f64]) -> bool {
    shares.len() >= 2
        && shares
            .iter()
            .all(|share| share.is_finite() && (0.0..=100.0).contains(share))
        && (shares.iter().sum::<f64>() - 100.0).abs() < 0.0001
}

fn action_index(id: ActionId) -> usize {
    match id {
        ActionId::PolicyDelivery => 0,
        ActionId::CrisisResponse => 1,
        ActionId::TransparencyReport => 2,
        ActionId::PublicServices => 3,
        ActionId::CommunityTour => 4,
        ActionId::PolicyForum => 5,
        ActionId::LegalAssembly => 6,
        ActionId::CoalitionTalks => 7,
        ActionId::OversightPetition => 8,
        ActionId::PublicMessage => 9,
        ActionId::RestAndPrepare => 10,
        ActionId::PollCoordination => 11,
    }
}

fn local_slate(region: &RegionProfile) -> ([&'static str; 3], [f64; 3]) {
    match region.previous_winner_party {
        "民進黨" => (
            ["民主進步黨", "中國國民黨", "台灣民眾黨"],
            [43.0, 37.0, 20.0],
        ),
        "民眾黨" => (
            ["台灣民眾黨", "民主進步黨", "中國國民黨"],
            [39.0, 33.0, 28.0],
        ),
        "無黨籍" => (["無黨籍", "中國國民黨", "民主進步黨"], [38.0, 34.0, 28.0]),
        _ => (
            ["中國國民黨", "民主進步黨", "台灣民眾黨"],
            [43.0, 37.0, 20.0],
        ),
    }
}

fn make_voter_segments(polls: &[f64]) -> Vec<VoterSegmentPoll> {
    VOTER_SEGMENTS
        .iter()
        .map(|segment| VoterSegmentPoll {
            name: segment.name,
            weight_pct: segment.weight_pct,
            polls: polls.to_vec(),
        })
        .collect()
}

fn shift_share(shares: &mut [f64], candidate_index: usize, delta: f64) {
    if shares.len() < 2 || candidate_index >= shares.len() || delta.abs() < f64::EPSILON {
        return;
    }
    let old = shares[candidate_index];
    let new = (old + delta).clamp(0.0, 100.0);
    let actual = new - old;
    let others_total = (100.0 - old).max(0.0);
    shares[candidate_index] = new;

    if actual > 0.0 {
        if others_total > f64::EPSILON {
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share -= actual * *share / others_total;
                }
            }
        }
    } else if actual < 0.0 {
        if others_total > f64::EPSILON {
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share += (-actual) * *share / others_total;
                }
            }
        } else {
            let each = (-actual) / (shares.len() - 1) as f64;
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share += each;
                }
            }
        }
    }
    normalize(shares);
}

fn transfer_share(shares: &mut [f64], from: usize, to: usize, requested: f64) {
    if from >= shares.len() || to >= shares.len() || from == to || requested <= 0.0 {
        return;
    }
    let amount = requested.min(shares[from]);
    shares[from] -= amount;
    shares[to] += amount;
    normalize(shares);
}

fn normalize(shares: &mut [f64]) {
    let total: f64 = shares.iter().sum();
    if total <= f64::EPSILON {
        return;
    }
    for share in shares {
        *share = (*share / total * 100.0).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_actions_receive_funding_only_at_week_end() {
        let mut game = GameState::new(2024, Side::Incumbent, 123);
        let before = game.budget;
        let outcome = game.apply_action(ActionId::PolicyDelivery).unwrap();
        assert!(game.validate().is_ok());
        assert_eq!(game.budget, before - 3);
        assert_eq!(outcome.week, 1);
        assert_eq!(outcome.day, 1);
        assert_eq!(outcome.weekly_funding, 0);
        for _ in 1..DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.round, 1);
        assert_eq!(game.day, 0);
        assert_eq!(game.budget, before - 3 + WEEKLY_FUNDING);
        assert!(game.polls.iter().all(|share| *share >= 0.0));
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn challenger_can_use_legal_assembly_and_grow_turnout() {
        let mut game = GameState::new(2016, Side::Challenger, 7);
        let before = game.turnout_momentum;
        game.apply_action(ActionId::LegalAssembly).unwrap();
        assert!(game.turnout_momentum > before);
    }

    #[test]
    fn same_seed_produces_the_same_campaign_and_final_draw() {
        let mut left = GameState::new(2024, Side::Challenger, 42);
        let mut right = GameState::new(2024, Side::Challenger, 42);
        for id in [
            ActionId::PolicyForum,
            ActionId::LegalAssembly,
            ActionId::CommunityTour,
        ] {
            left.apply_action(id).unwrap();
            right.apply_action(id).unwrap();
        }
        assert_eq!(left.polls, right.polls);
        assert_eq!(left.finish().shares, right.finish().shares);
    }

    #[test]
    fn explicit_default_model_parameters_preserve_the_existing_path() {
        let mut baseline =
            GameState::new_with_style(2024, Side::Challenger, 42, CampaignStyle::Policy);
        let mut configured = GameState::new_with_style_and_parameters(
            2024,
            Side::Challenger,
            42,
            CampaignStyle::Policy,
            ModelParameters::default(),
        );
        for action in [
            ActionId::PolicyForum,
            ActionId::LegalAssembly,
            ActionId::CommunityTour,
        ] {
            baseline.apply_action(action).unwrap();
            configured.apply_action(action).unwrap();
        }
        assert_eq!(baseline.polls, configured.polls);
        assert_eq!(baseline.finish(), configured.finish());
    }

    #[test]
    fn action_preview_is_non_mutating_and_does_not_consume_randomness() {
        let mut previewed =
            GameState::new_with_style(2024, Side::Challenger, 421, CampaignStyle::Policy);
        let mut control =
            GameState::new_with_style(2024, Side::Challenger, 421, CampaignStyle::Policy);
        let polls_before = previewed.polls.clone();
        let budget_before = previewed.budget;

        let preview = previewed.preview_action(ActionId::PolicyForum).unwrap();
        assert!(preview.expected_support_delta.is_finite());
        assert_eq!(previewed.polls, polls_before);
        assert_eq!(previewed.budget, budget_before);

        let with_preview = previewed.apply_action(ActionId::PolicyForum).unwrap();
        let without_preview = control.apply_action(ActionId::PolicyForum).unwrap();
        assert_eq!(previewed.polls, control.polls);
        assert_eq!(with_preview.share_after, without_preview.share_after);
        assert_eq!(
            with_preview.expected_support_delta,
            preview.expected_support_delta
        );
    }

    #[test]
    fn one_factor_multiplier_changes_only_the_selected_model_effect() {
        let reduced_action_effect = ModelParameters::default()
            .with_multiplier(SensitivityParameter::ActionSwing, 0.8)
            .unwrap();
        assert!(ModelParameters::default()
            .with_multiplier(SensitivityParameter::ActionSwing, 3.1)
            .is_err());
        assert!(ModelParameters::default()
            .with_multiplier(SensitivityParameter::DailyNoise, f64::NAN)
            .is_err());

        let mut baseline =
            GameState::new_with_style(2024, Side::Challenger, 73, CampaignStyle::Balanced);
        let mut reduced = GameState::new_with_style_and_parameters(
            2024,
            Side::Challenger,
            73,
            CampaignStyle::Balanced,
            reduced_action_effect,
        );
        baseline.apply_action(ActionId::PolicyForum).unwrap();
        reduced.apply_action(ActionId::PolicyForum).unwrap();
        assert_ne!(baseline.polls, reduced.polls);
    }

    #[test]
    fn weekly_event_schedule_is_seeded_and_avoids_adjacent_repeats() {
        let left = GameState::new(2024, Side::Challenger, 4242);
        let right = GameState::new(2024, Side::Challenger, 4242);
        let left_titles: Vec<_> = left
            .event_schedule
            .iter()
            .map(|event| event.title)
            .collect();
        let right_titles: Vec<_> = right
            .event_schedule
            .iter()
            .map(|event| event.title)
            .collect();

        assert_eq!(left_titles, right_titles);
        assert_eq!(left.event_schedule.len(), CAMPAIGN_ROUNDS as usize);
        assert!(left_titles.windows(2).all(|pair| pair[0] != pair[1]));
        assert!(left.event_schedule.iter().all(|event| left
            .events
            .iter()
            .any(|eligible| eligible.title == event.title)));
        assert!(left.validate().is_ok());
    }

    #[test]
    fn weekly_objective_schedule_is_seeded_and_varied() {
        let left = GameState::new(2024, Side::Challenger, 4242);
        let right = GameState::new(2024, Side::Challenger, 4242);
        let left_titles: Vec<_> = left
            .weekly_objective_schedule
            .iter()
            .map(|objective| objective.title)
            .collect();
        let right_titles: Vec<_> = right
            .weekly_objective_schedule
            .iter()
            .map(|objective| objective.title)
            .collect();

        assert_eq!(left_titles, right_titles);
        assert_eq!(left_titles.len(), CAMPAIGN_ROUNDS as usize);
        assert!(left_titles.windows(2).all(|pair| pair[0] != pair[1]));
        assert!(left.validate().is_ok());
    }

    #[test]
    fn weekly_objective_rewards_once_and_resets_at_week_boundary() {
        let mut game = GameState::new(2024, Side::Challenger, 77);
        let objective = game.current_objective().unwrap();
        let title = objective.title;
        let action = objective.qualifying_actions[0];

        game.apply_action(action).unwrap();
        assert_eq!(game.current_objective_progress(), 1);
        assert_eq!(game.objectives_completed(), 0);
        let completed = game.apply_action(action).unwrap();
        assert!(completed.objective_completed);
        assert_eq!(completed.objective_title, Some(title));
        assert_eq!(completed.objective_progress, objective.target_actions);
        assert_eq!(completed.objective_reward, 1);
        assert_eq!(game.objectives_completed(), 1);
        assert!(game.validate().is_ok());

        let repeated = game.apply_action(action).unwrap();
        assert!(!repeated.objective_completed);
        assert_eq!(repeated.objective_reward, 0);
        for _ in 0..DAYS_PER_WEEK - 3 {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.round, 1);
        assert_eq!(game.current_objective_progress(), 0);
        assert_ne!(game.current_objective().unwrap().title, title);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn campaign_phase_tracks_four_week_arcs_and_completion() {
        let mut game = GameState::new(2024, Side::Challenger, 3);
        assert_eq!(game.phase(), CampaignPhase::Foundation);
        for _ in 0..4 * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.phase(), CampaignPhase::IssueCompetition);
        for _ in 0..4 * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.phase(), CampaignPhase::GetOutTheVote);
        for _ in 0..4 * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.phase(), CampaignPhase::Complete);
    }

    #[test]
    fn settlement_is_idempotent_and_closes_the_engine_to_more_actions() {
        let mut game = GameState::new(2024, Side::Challenger, 314);
        let first = game.finish();
        let second = game.finish();
        assert_eq!(first, second);
        assert!(game.is_settled());
        assert!(game.validate().is_ok());
        assert!(matches!(
            game.apply_action(ActionId::CommunityTour),
            Err(GameError::AlreadySettled)
        ));
    }

    #[test]
    fn twelve_week_limit_is_enforced_after_eighty_four_daily_actions() {
        let mut game = GameState::new(2020, Side::Challenger, 1);
        for _ in 0..CAMPAIGN_ROUNDS * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert!(game.is_finished());
        assert!(game.apply_action(ActionId::CommunityTour).is_err());
    }

    #[test]
    fn resting_recovers_stamina_without_poll_swing() {
        let mut game = GameState::new(2024, Side::Challenger, 99);
        game.stamina = 40.0;
        let polls_before = game.polls.clone();
        let result = game.apply_action(ActionId::RestAndPrepare).unwrap();
        assert_eq!(game.polls, polls_before);
        assert_eq!(result.stamina_after, 43.5);
    }

    #[test]
    fn local_scenario_uses_region_context_and_three_party_polls() {
        let region = crate::history::find_region("臺北市").unwrap();
        let game = GameState::new_local_with_style(
            2026,
            Side::Challenger,
            15,
            CampaignStyle::Organizer,
            region,
        );
        assert_eq!(game.region.unwrap().name, "臺北市");
        assert_eq!(game.candidates.len(), 3);
        assert_eq!(game.player_index, 1);
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
    }

    #[test]
    fn multi_candidate_poll_coordination_produces_a_report() {
        let region = crate::history::find_region("新竹市").unwrap();
        let mut game = GameState::new_local_with_style(
            2026,
            Side::Challenger,
            13,
            CampaignStyle::Communicator,
            region,
        );
        let outcome = game.apply_action(ActionId::PollCoordination).unwrap();
        assert!(outcome.coordination_note.is_some());
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
        assert!(game.apply_action(ActionId::PollCoordination).is_err());
    }

    #[test]
    fn third_party_candidate_can_negotiate_a_coalition_once_per_week() {
        let region = crate::history::find_region("臺北市").unwrap();
        let party = crate::history::find_party("新黨").unwrap();
        let mut game = GameState::new_local_with_party(
            2026,
            Side::Challenger,
            888,
            CampaignStyle::Organizer,
            region,
            Some(party),
        );
        assert_eq!(game.candidates[game.player_index].party, "新黨");
        let outcome = game.apply_action(ActionId::CoalitionTalks).unwrap();
        assert!(outcome.coordination_note.is_some());
        assert!(game.apply_action(ActionId::CoalitionTalks).is_err());
    }

    #[test]
    fn voter_segment_weights_sum_to_one_hundred_and_update() {
        let mut game = GameState::new(2024, Side::Challenger, 17);
        let weight: f64 = game
            .voter_segments
            .iter()
            .map(|segment| segment.weight_pct)
            .sum();
        assert!((weight - 100.0).abs() < 0.001);
        game.apply_action(ActionId::PolicyForum).unwrap();
        assert!(game
            .voter_segments
            .iter()
            .any(
                |segment| (segment.polls[game.player_index] - game.polls[game.player_index]).abs()
                    > 0.01
            ));
    }
}
