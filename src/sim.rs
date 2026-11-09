use crate::campaign::{anniversary_day_offset, CampaignSetup, EventTimeline, DATED_EVENTS};
use crate::game_content::{
    baseline_election, challenger_candidate_index, events_for_campaign, find_party,
    incumbent_candidate_index, party_affinity, ElectionScenario, EventCard, Issue, PartyProfile,
    RegionProfile, VOTER_SEGMENTS,
};
use crate::governance::GovernmentOpinion;
use crate::staffer::{
    brief_for_round as staffer_brief_for_round, continuity_bonus as staffer_continuity_bonus,
    StafferBrief, StafferDecision, StafferProgress,
};
use crate::story::{
    beat_for_round, choice_for, coherence_bonus, StoryBeat, StoryDecision, StoryProgress,
};
use std::fmt;

pub const CAMPAIGN_ROUNDS: u8 = 12;
pub const DAYS_PER_WEEK: u8 = 7;
const DEFAULT_SEED: u64 = 0x0054_574E_5F53_494D;
const MAX_BUDGET: i32 = 120;

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
    HeadquartersAlreadyEstablished,
    SupportersAssociationAlreadyEstablished,
    StoryChoiceUnavailable,
    StafferDecisionUnavailable,
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
            Self::HeadquartersAlreadyEstablished => {
                formatter.write_str("競選總部已成立；每場戰役只能成立一次。")
            }
            Self::SupportersAssociationAlreadyEstablished => {
                formatter.write_str("後援會已成立；每場戰役只能成立一次。")
            }
            Self::StoryChoiceUnavailable => {
                formatter.write_str("目前沒有可處理的故事情境，或所選解法無效。")
            }
            Self::StafferDecisionUnavailable => {
                formatter.write_str("目前沒有可處理的幕僚會議，或所選方案無效。")
            }
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
    CampaignHeadquarters,
    VolunteerCanvassing,
    GroundTeamLeaflets,
    StreetSweep,
    TurnoutMobilization,
    Debate,
    MediaInterview,
    PolicyPresentation,
    FactCheckResponse,
    FabricatedPoll,
    AnonymousSmear,
    CoordinatedOpinionWar,
    SupportersAssociation,
    ReligiousCommunityVisit,
    PilgrimageParticipation,
    MassAttendance,
    VulnerableCommunityCare,
    DisasterResponseVisit,
}

impl ActionId {
    pub const COUNT: usize = 30;
    pub const ALL: [Self; Self::COUNT] = [
        Self::PolicyDelivery,
        Self::CrisisResponse,
        Self::TransparencyReport,
        Self::PublicServices,
        Self::CommunityTour,
        Self::PolicyForum,
        Self::LegalAssembly,
        Self::CoalitionTalks,
        Self::OversightPetition,
        Self::PublicMessage,
        Self::RestAndPrepare,
        Self::PollCoordination,
        Self::CampaignHeadquarters,
        Self::VolunteerCanvassing,
        Self::GroundTeamLeaflets,
        Self::StreetSweep,
        Self::TurnoutMobilization,
        Self::Debate,
        Self::MediaInterview,
        Self::PolicyPresentation,
        Self::FactCheckResponse,
        Self::FabricatedPoll,
        Self::AnonymousSmear,
        Self::CoordinatedOpinionWar,
        Self::SupportersAssociation,
        Self::ReligiousCommunityVisit,
        Self::PilgrimageParticipation,
        Self::MassAttendance,
        Self::VulnerableCommunityCare,
        Self::DisasterResponseVisit,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::PolicyDelivery => 0,
            Self::CrisisResponse => 1,
            Self::TransparencyReport => 2,
            Self::PublicServices => 3,
            Self::CommunityTour => 4,
            Self::PolicyForum => 5,
            Self::LegalAssembly => 6,
            Self::CoalitionTalks => 7,
            Self::OversightPetition => 8,
            Self::PublicMessage => 9,
            Self::RestAndPrepare => 10,
            Self::PollCoordination => 11,
            Self::CampaignHeadquarters => 12,
            Self::VolunteerCanvassing => 13,
            Self::GroundTeamLeaflets => 14,
            Self::StreetSweep => 15,
            Self::TurnoutMobilization => 16,
            Self::Debate => 17,
            Self::MediaInterview => 18,
            Self::PolicyPresentation => 19,
            Self::FactCheckResponse => 20,
            Self::FabricatedPoll => 21,
            Self::AnonymousSmear => 22,
            Self::CoordinatedOpinionWar => 23,
            Self::SupportersAssociation => 24,
            Self::ReligiousCommunityVisit => 25,
            Self::PilgrimageParticipation => 26,
            Self::MassAttendance => 27,
            Self::VulnerableCommunityCare => 28,
            Self::DisasterResponseVisit => 29,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::PolicyDelivery => "policy_delivery",
            Self::CrisisResponse => "crisis_response",
            Self::TransparencyReport => "transparency_report",
            Self::PublicServices => "public_services",
            Self::CommunityTour => "community_tour",
            Self::PolicyForum => "policy_forum",
            Self::LegalAssembly => "legal_assembly",
            Self::CoalitionTalks => "coalition_talks",
            Self::OversightPetition => "oversight_petition",
            Self::PublicMessage => "public_message",
            Self::RestAndPrepare => "rest_and_prepare",
            Self::PollCoordination => "poll_coordination",
            Self::CampaignHeadquarters => "campaign_headquarters",
            Self::VolunteerCanvassing => "volunteer_canvassing",
            Self::GroundTeamLeaflets => "ground_team_leaflets",
            Self::StreetSweep => "street_sweep",
            Self::TurnoutMobilization => "turnout_mobilization",
            Self::Debate => "debate",
            Self::MediaInterview => "media_interview",
            Self::PolicyPresentation => "policy_presentation",
            Self::FactCheckResponse => "fact_check_response",
            Self::FabricatedPoll => "fabricated_poll",
            Self::AnonymousSmear => "anonymous_smear",
            Self::CoordinatedOpinionWar => "coordinated_opinion_war",
            Self::SupportersAssociation => "supporters_association",
            Self::ReligiousCommunityVisit => "religious_community_visit",
            Self::PilgrimageParticipation => "pilgrimage_participation",
            Self::MassAttendance => "mass_attendance",
            Self::VulnerableCommunityCare => "vulnerable_community_care",
            Self::DisasterResponseVisit => "disaster_response_visit",
        }
    }

    pub fn category(self) -> TacticCategory {
        match self {
            Self::FabricatedPoll | Self::AnonymousSmear | Self::CoordinatedOpinionWar => {
                TacticCategory::Negative
            }
            Self::CoalitionTalks
            | Self::PollCoordination
            | Self::RestAndPrepare
            | Self::Debate
            | Self::MediaInterview
            | Self::PolicyPresentation
            | Self::FactCheckResponse
            | Self::ReligiousCommunityVisit
            | Self::PilgrimageParticipation
            | Self::MassAttendance => TacticCategory::Neutral,
            _ => TacticCategory::Positive,
        }
    }

    fn base_exposure_risk(self) -> Option<f64> {
        match self {
            Self::FabricatedPoll => Some(0.24),
            Self::AnonymousSmear => Some(0.32),
            Self::CoordinatedOpinionWar => Some(0.20),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TacticCategory {
    Positive,
    Neutral,
    Negative,
}

impl TacticCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::Positive => "正面選戰",
            Self::Neutral => "中立選戰",
            Self::Negative => "負面／高風險選戰",
        }
    }
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
            ActionId::PolicyPresentation,
        ],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "公開資訊與回應質疑",
        description: "透過施政檢討或公開政策問答完成兩項溝通行動。",
        qualifying_actions: &[
            ActionId::TransparencyReport,
            ActionId::PublicMessage,
            ActionId::MediaInterview,
            ActionId::FactCheckResponse,
        ],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "穩定地方服務",
        description: "以民生服務或施政方案回應在地生活議題，完成兩項行動。",
        qualifying_actions: &[
            ActionId::PublicServices,
            ActionId::PolicyDelivery,
            ActionId::VulnerableCommunityCare,
            ActionId::DisasterResponseVisit,
        ],
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
        qualifying_actions: &[
            ActionId::CommunityTour,
            ActionId::LegalAssembly,
            ActionId::VolunteerCanvassing,
            ActionId::GroundTeamLeaflets,
            ActionId::StreetSweep,
            ActionId::TurnoutMobilization,
            ActionId::ReligiousCommunityVisit,
            ActionId::VulnerableCommunityCare,
            ActionId::DisasterResponseVisit,
        ],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "提出可檢驗的替代方案",
        description: "以政見論壇或監督提案完成兩項政策論述行動。",
        qualifying_actions: &[
            ActionId::PolicyForum,
            ActionId::OversightPetition,
            ActionId::PolicyPresentation,
            ActionId::Debate,
        ],
        target_actions: 2,
    },
    WeeklyObjective {
        title: "建立可信的監督與說明",
        description: "用監督提案或公開政策問答完成兩項溝通行動。",
        qualifying_actions: &[
            ActionId::OversightPetition,
            ActionId::PublicMessage,
            ActionId::FactCheckResponse,
            ActionId::MediaInterview,
        ],
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

const COMMON_CAMPAIGN_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: ActionId::CampaignHeadquarters,
        name: "成立競選總部與志工調度",
        description: "投入場地與人力建立一次性組織基礎，後續地面行動可受益。",
        cost: 8,
        base_swing: 0.0,
        trust_delta: 0.05,
        turnout_delta: 0.08,
        issues: &[Issue::Governance, Issue::Civic, Issue::Local],
        stamina_cost: 0.5,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::VolunteerCanvassing,
        name: "志工組織與逐戶拜票",
        description: "組織志工接觸社區、聆聽需求並回報地方議題。",
        cost: 2,
        base_swing: 0.26,
        trust_delta: 0.20,
        turnout_delta: 0.18,
        issues: &[Issue::Local, Issue::Rights, Issue::Education],
        stamina_cost: 1.3,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::GroundTeamLeaflets,
        name: "地面競選人員掃街與發文宣",
        description: "以公開競選人力進行街頭接觸與政見文宣發放。",
        cost: 3,
        base_swing: 0.22,
        trust_delta: 0.12,
        turnout_delta: 0.18,
        issues: &[Issue::Local, Issue::Civic, Issue::Media],
        stamina_cost: 1.7,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::StreetSweep,
        name: "候選人掃街與市場拜訪",
        description: "以公開行程接觸居民、商家與地方社群。",
        cost: 2,
        base_swing: 0.26,
        trust_delta: 0.18,
        turnout_delta: 0.21,
        issues: &[Issue::Local, Issue::Civic, Issue::Trust],
        stamina_cost: 1.7,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::TurnoutMobilization,
        name: "支持者動員與投票提醒",
        description: "整合公開志工與支持者聯繫，提升投票參與動能。",
        cost: 3,
        base_swing: 0.15,
        trust_delta: 0.02,
        turnout_delta: 0.48,
        issues: &[Issue::Civic, Issue::Rights],
        stamina_cost: 1.2,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::Debate,
        name: "電視辯論與政策攻防",
        description: "在公開辯論中比較方案、回應質疑並接受檢驗。",
        cost: 1,
        base_swing: 0.18,
        trust_delta: 0.22,
        turnout_delta: 0.06,
        issues: &[Issue::Economy, Issue::Governance, Issue::CrossStrait],
        stamina_cost: 1.1,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::MediaInterview,
        name: "媒體專訪",
        description: "接受採訪說明立場、回答追問並釐清爭議。",
        cost: 0,
        base_swing: 0.12,
        trust_delta: 0.18,
        turnout_delta: 0.03,
        issues: &[Issue::Media, Issue::Trust],
        stamina_cost: 0.5,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PolicyPresentation,
        name: "政見發表會",
        description: "公開提出可檢驗的政策主張、執行期程與資源安排。",
        cost: 2,
        base_swing: 0.29,
        trust_delta: 0.25,
        turnout_delta: 0.06,
        issues: &[
            Issue::Economy,
            Issue::Governance,
            Issue::PublicHealth,
            Issue::Local,
        ],
        stamina_cost: 1.2,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::FactCheckResponse,
        name: "查證與公開澄清",
        description: "公布可核對資料、說明不確定處並回應錯誤訊息。",
        cost: 1,
        base_swing: 0.14,
        trust_delta: 0.25,
        turnout_delta: 0.05,
        issues: &[Issue::Media, Issue::Trust, Issue::Governance],
        stamina_cost: 0.7,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::FabricatedPoll,
        name: "造假民調／誤導性數字操作",
        description: "短期聲量可能增加，但有被查證揭露、信任受損與罰款風險。",
        cost: 1,
        base_swing: 0.38,
        trust_delta: 0.0,
        turnout_delta: 0.03,
        issues: &[Issue::Trust, Issue::Media],
        stamina_cost: 0.4,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::AnonymousSmear,
        name: "匿名黑函／未查證指控",
        description: "短期可能轉移議題焦點；查證後將觸發較重的信任與資源懲處。",
        cost: 1,
        base_swing: 0.42,
        trust_delta: -0.02,
        turnout_delta: 0.01,
        issues: &[Issue::Governance, Issue::Trust, Issue::Media],
        stamina_cost: 0.5,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CoordinatedOpinionWar,
        name: "協調式網路輿論操作",
        description: "高風險聲量操作；遭揭露時會受到支持度、信任與預算懲處。",
        cost: 2,
        base_swing: 0.33,
        trust_delta: -0.04,
        turnout_delta: 0.12,
        issues: &[Issue::Civic, Issue::Media, Issue::Trust],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::SupportersAssociation,
        name: "成立後援會與地方聯絡網",
        description: "建立公開、可問責的支持者組織，增加後續地面行動的調度量能。",
        cost: 5,
        base_swing: 0.0,
        trust_delta: 0.08,
        turnout_delta: 0.06,
        issues: &[Issue::Local, Issue::Civic, Issue::Governance],
        stamina_cost: 0.6,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::ReligiousCommunityVisit,
        name: "拜訪宗教團體與在地社群",
        description: "公開聆聽團體關注，不以信仰身分推定政治立場或支持。",
        cost: 1,
        base_swing: 0.12,
        trust_delta: 0.14,
        turnout_delta: 0.05,
        issues: &[Issue::Local, Issue::Civic, Issue::Trust],
        stamina_cost: 0.8,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PilgrimageParticipation,
        name: "參與公開遶境行程",
        description: "以尊重儀式與地方自治的方式參與公開活動，效果依情境而異。",
        cost: 1,
        base_swing: 0.10,
        trust_delta: 0.12,
        turnout_delta: 0.06,
        issues: &[Issue::Local, Issue::Civic],
        stamina_cost: 1.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::MassAttendance,
        name: "出席公開彌撒或宗教活動",
        description: "以公開且尊重信仰自主的方式出席，不預設信眾政治傾向。",
        cost: 0,
        base_swing: 0.08,
        trust_delta: 0.10,
        turnout_delta: 0.04,
        issues: &[Issue::Local, Issue::Civic, Issue::Trust],
        stamina_cost: 0.7,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::VulnerableCommunityCare,
        name: "關懷弱勢與社區服務",
        description: "投入可持續的社區服務並連結公共資源，不以單次曝光取代政策。",
        cost: 3,
        base_swing: 0.24,
        trust_delta: 0.23,
        turnout_delta: 0.10,
        issues: &[Issue::PublicHealth, Issue::Local, Issue::Rights],
        stamina_cost: 1.1,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::DisasterResponseVisit,
        name: "勘災與災後關懷",
        description: "掌握受災需求、協調救助資訊並追蹤後續；議題契合時較能回應民眾關切。",
        cost: 4,
        base_swing: 0.16,
        trust_delta: 0.30,
        turnout_delta: 0.08,
        issues: &[Issue::PublicHealth, Issue::Local, Issue::Governance],
        stamina_cost: 1.2,
        stamina_recovery: 0.0,
    },
];

pub fn actions_for(side: Side) -> &'static [ActionSpec] {
    match side {
        Side::Incumbent => INCUMBENT_ACTIONS,
        Side::Challenger => CHALLENGER_ACTIONS,
    }
}

/// All currently available operations for a side, including shared campaign tactics.
pub fn campaign_actions_for(side: Side) -> Vec<ActionSpec> {
    let mut actions = actions_for(side).to_vec();
    actions.extend_from_slice(COMMON_CAMPAIGN_ACTIONS);
    actions
}

fn find_campaign_action(side: Side, id: ActionId) -> Option<ActionSpec> {
    actions_for(side)
        .iter()
        .chain(COMMON_CAMPAIGN_ACTIONS)
        .find(|action| action.id == id)
        .copied()
}

#[derive(Clone, Debug)]
pub struct SimCandidate {
    pub name: String,
    pub party: &'static str,
    pub starting_share: f64,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub campaign_year: u16,
    pub baseline_year: u16,
    pub region: Option<&'static RegionProfile>,
    pub side: Side,
    pub style: CampaignStyle,
    pub setup: CampaignSetup,
    model_parameters: ModelParameters,
    pub player_attributes: CandidateAttributes,
    pub player_index: usize,
    pub candidates: Vec<SimCandidate>,
    pub polls: Vec<f64>,
    /// Separate in-game administration and issue ratings; not candidate polls.
    pub government_opinion: GovernmentOpinion,
    pub voter_segments: Vec<VoterSegmentPoll>,
    pub budget: i32,
    pub headquarters_established: bool,
    pub supporters_association_established: bool,
    pub field_capacity: f64,
    pub exposed_tactics: u8,
    pub sanctioned_funds: i32,
    pub trust: f64,
    pub turnout_momentum: f64,
    pub stamina: f64,
    pub round: u8,
    pub day: u8,
    pub events: Vec<&'static EventCard>,
    /// One seeded issue focus per campaign week. `events` remains the full
    /// eligible event pool displayed to the player.
    event_schedule: Vec<Option<&'static EventCard>>,
    weekly_objective_schedule: Vec<&'static WeeklyObjective>,
    weekly_objective_progress: u8,
    weekly_objective_completed: bool,
    objectives_completed: u8,
    action_uses: [u8; ActionId::COUNT],
    matched_events: u8,
    pub poll_taken_this_week: bool,
    pub coalition_this_week: bool,
    pub story_progress: StoryProgress,
    pub staffer_progress: StafferProgress,
    final_outcome: Option<ElectionOutcome>,
    rng: Rng,
}

#[derive(Clone, Debug)]
pub struct ActionOutcome {
    pub id: ActionId,
    pub action_name: &'static str,
    pub category: TacticCategory,
    pub rival_response: Option<RivalResponse>,
    pub event_title: Option<&'static str>,
    pub share_before: f64,
    pub share_after: f64,
    pub expected_support_delta: f64,
    pub detection_probability: f64,
    pub negative_tactic_detected: bool,
    pub sanction_fine: i32,
    pub exposed_tactics: u8,
    pub sanctioned_funds: i32,
    pub field_capacity_after: f64,
    pub tactic_note: Option<String>,
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

#[derive(Clone, Debug, PartialEq)]
pub struct RivalResponse {
    pub candidate_name: String,
    pub share_before: f64,
    pub share_after: f64,
}

#[derive(Clone, Debug)]
pub struct StoryOutcome {
    pub title: &'static str,
    pub choice_label: &'static str,
    pub support_before: f64,
    pub support_after: f64,
    pub support_delta: f64,
    pub trust_after: f64,
    pub mobilization_after: f64,
    pub stamina_after: f64,
    pub budget_after: i32,
    pub note: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub struct StafferOutcome {
    pub title: &'static str,
    pub choice_label: &'static str,
    pub support_before: f64,
    pub support_after: f64,
    pub support_delta: f64,
    pub trust_after: f64,
    pub mobilization_after: f64,
    pub stamina_after: f64,
    pub budget_after: i32,
    pub event_match: bool,
    pub note: Option<&'static str>,
}

/// A deterministic, one-step estimate of an action before seeded noise is drawn.
/// It is a forecast from this simulation model, not a poll or election prediction.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionPreview {
    pub id: ActionId,
    pub action_name: &'static str,
    pub action_description: &'static str,
    pub category: TacticCategory,
    pub expected_swing: f64,
    pub direct_support_delta: f64,
    pub coordination_support_delta: f64,
    pub expected_response_delta: f64,
    pub expected_support_delta: f64,
    pub detection_probability: f64,
    pub expected_sanction_fine: f64,
    pub infrastructure_delta: f64,
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
    /// Replace the scenario slate before play begins. Parties must belong to
    /// the game's fictional registry; personal support values are normalized.
    pub fn apply_candidate_roster(
        &mut self,
        entries: &[(String, &'static str, f64)],
        player_index: usize,
    ) -> Result<(), &'static str> {
        if self.round != 0 || self.day != 0 || self.final_outcome.is_some() {
            return Err("candidate rosters can only be applied before the first campaign day");
        }
        if !(2..=5).contains(&entries.len()) || player_index >= entries.len() {
            return Err("candidate roster must contain 2..=5 candidates and a valid player slot");
        }
        if entries.iter().any(|(name, party, share)| {
            name.trim().chars().count() < 2
                || name.trim().chars().count() > 24
                || name.chars().any(char::is_control)
                || party.is_empty()
                || find_party(party).is_none()
                || !share.is_finite()
                || !(0.0..=100.0).contains(share)
        }) {
            return Err("candidate roster contains an invalid name, party, or support value");
        }
        let total: f64 = entries.iter().map(|entry| entry.2).sum();
        if !total.is_finite() || total <= 0.0 {
            return Err("candidate roster must have a positive finite support total");
        }
        let normalized: Vec<f64> = entries
            .iter()
            .map(|entry| entry.2 / total * 100.0)
            .collect();
        self.candidates = entries
            .iter()
            .zip(&normalized)
            .map(|((name, party, _), share)| SimCandidate {
                name: name.clone(),
                party,
                starting_share: *share,
            })
            .collect();
        self.polls = normalized;
        self.player_index = player_index;
        self.voter_segments = make_voter_segments(&self.polls);
        self.final_outcome = None;
        self.validate()
    }

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
        Self::new_with_setup(
            campaign_year,
            side,
            seed,
            style,
            model_parameters,
            CampaignSetup::default(),
        )
    }

    pub fn new_with_setup(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        model_parameters: ModelParameters,
        setup: CampaignSetup,
    ) -> Self {
        let election = baseline_election(campaign_year);
        Self::from_election(
            campaign_year,
            side,
            election,
            seed,
            style,
            model_parameters,
            setup,
        )
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
        Self::new_local_with_party_and_setup(
            campaign_year,
            side,
            seed,
            style,
            region,
            selected_party,
            model_parameters,
            CampaignSetup::default(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_local_with_party_and_setup(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        region: &'static RegionProfile,
        selected_party: Option<&'static PartyProfile>,
        model_parameters: ModelParameters,
        setup: CampaignSetup,
    ) -> Self {
        let (mut parties, shares) = local_slate(region);
        if let Some(selected_party) = selected_party {
            if !parties.contains(&selected_party.name) {
                parties[2] = selected_party.name;
            }
        }
        let candidates = parties
            .iter()
            .zip(shares)
            .map(|(party, share)| SimCandidate {
                name: format!("{party}（地方情境代表）"),
                party,
                starting_share: share,
            })
            .collect();
        let player_index = match side {
            Side::Incumbent => 0,
            Side::Challenger => selected_party
                .and_then(|party| parties.iter().position(|name| *name == party.name))
                .unwrap_or(1),
        };
        Self::from_candidates(
            campaign_year,
            2024,
            side,
            seed,
            style,
            model_parameters,
            setup,
            Some(region),
            candidates,
            player_index,
            "2026-10-24",
        )
    }

    fn from_election(
        campaign_year: u16,
        side: Side,
        election: &'static ElectionScenario,
        seed: u64,
        style: CampaignStyle,
        model_parameters: ModelParameters,
        setup: CampaignSetup,
    ) -> Self {
        let player_index = match side {
            Side::Incumbent => incumbent_candidate_index(election),
            Side::Challenger => challenger_candidate_index(election),
        };
        let uses_scenario_names = campaign_year == election.year;
        let candidates: Vec<SimCandidate> = election
            .candidates
            .iter()
            .map(|result| SimCandidate {
                name: if uses_scenario_names {
                    format!("{} / {}", result.president, result.vice_president)
                } else {
                    format!("{}（情境代表）", result.party)
                },
                party: result.party,
                starting_share: result.share_pct,
            })
            .collect();
        Self::from_candidates(
            campaign_year,
            election.year,
            side,
            seed,
            style,
            model_parameters,
            setup,
            None,
            candidates,
            player_index,
            election.date,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_candidates(
        campaign_year: u16,
        baseline_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        model_parameters: ModelParameters,
        setup: CampaignSetup,
        region: Option<&'static RegionProfile>,
        candidates: Vec<SimCandidate>,
        player_index: usize,
        baseline_date: &'static str,
    ) -> Self {
        let mut polls: Vec<f64> = candidates
            .iter()
            .map(|candidate| candidate.starting_share)
            .collect();
        normalize(&mut polls);
        let (trust, turnout_momentum) = match side {
            Side::Incumbent => (54.0, 50.0),
            Side::Challenger => (50.0, 50.0),
        };
        let voter_segments = make_voter_segments(&polls);
        let all_events = events_for_campaign(campaign_year);
        let events = setup.events(&all_events);
        let mut rng = Rng::new(seed);
        let event_schedule = match setup.timeline {
            EventTimeline::SeededWeekly => make_event_schedule(&events, &mut rng)
                .into_iter()
                .map(Some)
                .collect(),
            EventTimeline::AnniversaryMapped => {
                let election_date = if campaign_year == baseline_year {
                    baseline_date
                } else if campaign_year == 2026 {
                    "2026-10-24"
                } else {
                    baseline_date
                };
                make_anniversary_event_schedule(&events, election_date)
            }
        };
        let weekly_objective_schedule = make_weekly_objective_schedule(side, &mut rng);

        Self {
            campaign_year,
            baseline_year,
            region,
            side,
            style,
            setup,
            model_parameters,
            player_attributes: style.attributes(),
            player_index,
            candidates,
            polls,
            government_opinion: GovernmentOpinion::default(),
            voter_segments,
            budget: setup.difficulty.starting_budget(),
            headquarters_established: false,
            supporters_association_established: false,
            field_capacity: 0.0,
            exposed_tactics: 0,
            sanctioned_funds: 0,
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
            action_uses: [0; ActionId::COUNT],
            matched_events: 0,
            poll_taken_this_week: false,
            coalition_this_week: false,
            story_progress: StoryProgress::default(),
            staffer_progress: StafferProgress::default(),
            final_outcome: None,
            rng,
        }
    }

    pub fn current_event(&self) -> Option<&'static EventCard> {
        self.event_schedule
            .get(self.round as usize)
            .copied()
            .flatten()
    }

    pub fn current_objective(&self) -> Option<&'static WeeklyObjective> {
        self.weekly_objective_schedule
            .get(self.round as usize)
            .copied()
    }

    pub fn current_objective_progress(&self) -> u8 {
        self.weekly_objective_progress
    }

    pub fn current_story_beat(&self) -> Option<StoryBeat> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS {
            return None;
        }
        beat_for_round(self.round, self.story_progress)
    }

    /// Resolve an optional story choice without consuming one of the 84 daily turns.
    pub fn apply_story_choice(&mut self, choice_index: usize) -> Result<StoryOutcome, GameError> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS {
            return Err(GameError::StoryChoiceUnavailable);
        }
        let beat = self
            .current_story_beat()
            .ok_or(GameError::StoryChoiceUnavailable)?;
        let choice =
            *choice_for(beat.index, choice_index).ok_or(GameError::StoryChoiceUnavailable)?;
        if self.budget < choice.budget_cost {
            return Err(GameError::InsufficientBudget {
                required: choice.budget_cost,
                available: self.budget,
            });
        }
        let bonus = coherence_bonus(beat.index, choice.id, self.story_progress);
        let support_before = self.polls[self.player_index];
        if !self
            .story_progress
            .record(beat.index, StoryDecision::Chosen(choice.id))
        {
            return Err(GameError::StoryChoiceUnavailable);
        }
        self.budget -= choice.budget_cost;
        self.apply_segment_swing(
            self.player_index,
            choice.support_delta + bonus.support_delta,
            &[],
        );
        self.trust = (self.trust + choice.trust_delta + bonus.trust_delta).clamp(0.0, 100.0);
        self.turnout_momentum =
            (self.turnout_momentum + choice.mobilization_delta).clamp(0.0, 100.0);
        self.stamina = (self.stamina + choice.stamina_delta).clamp(0.0, 100.0);
        let support_after = self.polls[self.player_index];
        Ok(StoryOutcome {
            title: beat.title,
            choice_label: choice.label,
            support_before,
            support_after,
            support_delta: support_after - support_before,
            trust_after: self.trust,
            mobilization_after: self.turnout_momentum,
            stamina_after: self.stamina,
            budget_after: self.budget,
            note: bonus.note,
        })
    }

    pub fn skip_story_beat(&mut self) -> Result<(), GameError> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS {
            return Err(GameError::StoryChoiceUnavailable);
        }
        let beat = self
            .current_story_beat()
            .ok_or(GameError::StoryChoiceUnavailable)?;
        if !self
            .story_progress
            .record(beat.index, StoryDecision::Skipped)
        {
            return Err(GameError::StoryChoiceUnavailable);
        }
        Ok(())
    }

    pub fn current_staffer_brief(&self) -> Option<StafferBrief> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS || self.day != 0 {
            return None;
        }
        staffer_brief_for_round(self.round, self.staffer_progress)
    }

    /// Resolve a staffer-mode planning choice without consuming one of the 84 daily turns.
    pub fn apply_staffer_choice(
        &mut self,
        choice_index: usize,
    ) -> Result<StafferOutcome, GameError> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS || self.day != 0 {
            return Err(GameError::StafferDecisionUnavailable);
        }
        let brief = self
            .current_staffer_brief()
            .ok_or(GameError::StafferDecisionUnavailable)?;
        let choice = *brief
            .choices
            .get(choice_index)
            .ok_or(GameError::StafferDecisionUnavailable)?;
        if self.budget < choice.budget_cost {
            return Err(GameError::InsufficientBudget {
                required: choice.budget_cost,
                available: self.budget,
            });
        }
        let continuity = staffer_continuity_bonus(brief.index, &choice, self.staffer_progress);
        let event_match = self
            .current_event()
            .is_some_and(|event| event.issue == choice.issue);
        let event_support_bonus = if event_match { 0.04 } else { 0.0 };
        let event_trust_bonus = if event_match { 0.2 } else { 0.0 };
        let support_before = self.polls[self.player_index];
        if !self.staffer_progress.record(
            brief.index,
            StafferDecision::Chosen(choice.id),
            choice.focus,
        ) {
            return Err(GameError::StafferDecisionUnavailable);
        }
        self.budget -= choice.budget_cost;
        self.apply_segment_swing(
            self.player_index,
            choice.support_delta + continuity.support_delta + event_support_bonus,
            &[],
        );
        self.trust = (self.trust + choice.trust_delta + continuity.trust_delta + event_trust_bonus)
            .clamp(0.0, 100.0);
        self.turnout_momentum =
            (self.turnout_momentum + choice.mobilization_delta).clamp(0.0, 100.0);
        self.stamina = (self.stamina + choice.stamina_delta).clamp(0.0, 100.0);
        self.government_opinion.observe(
            choice.issue,
            0.0,
            0.0,
            0.35 + if event_match { 0.2 } else { 0.0 },
        );
        let support_after = self.polls[self.player_index];
        let note = if continuity.note.is_some() {
            continuity.note
        } else if event_match {
            Some("方案呼應本週事件議題，獲得小幅遊戲內契合加成。")
        } else {
            None
        };
        Ok(StafferOutcome {
            title: brief.title,
            choice_label: choice.label,
            support_before,
            support_after,
            support_delta: support_after - support_before,
            trust_after: self.trust,
            mobilization_after: self.turnout_momentum,
            stamina_after: self.stamina,
            budget_after: self.budget,
            event_match,
            note,
        })
    }

    pub fn skip_staffer_brief(&mut self) -> Result<(), GameError> {
        if self.final_outcome.is_some() || self.round >= CAMPAIGN_ROUNDS || self.day != 0 {
            return Err(GameError::StafferDecisionUnavailable);
        }
        let brief = self
            .current_staffer_brief()
            .ok_or(GameError::StafferDecisionUnavailable)?;
        if !self
            .staffer_progress
            .record(brief.index, StafferDecision::Skipped, None)
        {
            return Err(GameError::StafferDecisionUnavailable);
        }
        Ok(())
    }

    fn action_spec(&self, id: ActionId) -> Result<ActionSpec, GameError> {
        if self.final_outcome.is_some() {
            return Err(GameError::AlreadySettled);
        }
        if self.round >= CAMPAIGN_ROUNDS {
            return Err(GameError::CampaignComplete);
        }
        let action = find_campaign_action(self.side, id).ok_or(GameError::ActionUnavailable)?;
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
        if id == ActionId::CampaignHeadquarters && self.headquarters_established {
            return Err(GameError::HeadquartersAlreadyEstablished);
        }
        if id == ActionId::SupportersAssociation && self.supporters_association_established {
            return Err(GameError::SupportersAssociationAlreadyEstablished);
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
        let detection_probability = self.detection_probability(action.id);
        let expected_sanction_fine = detection_probability
            * self
                .sanction_schedule()
                .min(self.budget.saturating_sub(action.cost)) as f64;
        let infrastructure_delta = self.infrastructure_gain(action.id);
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

        if detection_probability > 0.0 {
            apply_segment_swing_to_state(
                &mut projected_polls,
                &mut projected_segments,
                self.player_index,
                -1.6 * detection_probability,
                action.issues,
            );
        }

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
        let trust_after =
            (self.trust + action.trust_delta - 16.0 * detection_probability).clamp(0.0, 100.0);
        let field_turnout = if is_field_action(action.id) {
            self.field_capacity * 0.0015
        } else {
            0.0
        };
        let turnout_after = (self.turnout_momentum + action.turnout_delta + field_turnout
            - 0.16 * detection_probability)
            .clamp(0.0, 100.0);
        let stamina_after =
            (self.stamina - action.stamina_cost + action.stamina_recovery).clamp(0.0, 100.0);

        ActionPreview {
            id: action.id,
            action_name: action.name,
            action_description: action.description,
            category: action.id.category(),
            expected_swing,
            direct_support_delta,
            coordination_support_delta,
            expected_response_delta,
            expected_support_delta: expected_support_after - before,
            detection_probability,
            expected_sanction_fine,
            infrastructure_delta,
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
        let infrastructure_bonus = if is_field_action(action.id) {
            self.field_capacity * 0.0025
        } else {
            0.0
        };
        action.base_swing
            * self
                .model_parameters
                .multiplier(SensitivityParameter::ActionSwing)
            + issue_bonus
            + local_match
            + trust_bonus
            + skill_bonus
            + fatigue_penalty
            + infrastructure_bonus
    }

    fn detection_probability(&self, id: ActionId) -> f64 {
        let Some(base_risk) = id.base_exposure_risk() else {
            return 0.0;
        };
        let repeated_use = f64::from(self.action_uses[action_index(id)].min(4)) * 0.10;
        let previous_exposures = f64::from(self.exposed_tactics) * 0.08;
        (base_risk + repeated_use + previous_exposures).clamp(0.05, 0.90)
    }

    fn sanction_schedule(&self) -> i32 {
        (5 + i32::from(self.exposed_tactics) * 2).min(15)
    }

    fn infrastructure_gain(&self, id: ActionId) -> f64 {
        match id {
            ActionId::CampaignHeadquarters => 35.0,
            ActionId::SupportersAssociation => 20.0,
            _ => 0.0,
        }
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

        let mut negative_tactic_detected = false;
        let mut sanction_fine = 0;
        let tactic_note = if preview.detection_probability > 0.0 {
            negative_tactic_detected = self.rng.unit() < preview.detection_probability;
            if negative_tactic_detected {
                let scheduled_fine = self.sanction_schedule().min(self.budget);
                self.exposed_tactics = self.exposed_tactics.saturating_add(1);
                sanction_fine = scheduled_fine;
                self.budget -= sanction_fine;
                self.sanctioned_funds = self.sanctioned_funds.saturating_add(sanction_fine);
                self.apply_segment_swing(self.player_index, -1.6, action.issues);
                Some(format!(
                    "查核揭露此項負面操作：信任與支持受損，另遭遊戲內資源懲處 {}（累計 {}）。",
                    sanction_fine, self.sanctioned_funds
                ))
            } else {
                Some("本次未被查核揭露；後續重複操作會提高風險。".to_string())
            }
        } else {
            None
        };

        let observed_issue = action.issues.first().copied().unwrap_or(Issue::Governance);
        let event_issue = current_event.map(|event| event.issue);
        self.observe_government_action(id, observed_issue, event_issue, negative_tactic_detected);

        // A competing campaign responds to daily activity with a small seeded
        // movement; expose it as an in-game response, not as polling evidence.
        let rival_response = if id == ActionId::RestAndPrepare {
            None
        } else if let Some(rival_index) = self.strongest_rival() {
            let candidate_name = self.candidates[rival_index].name.clone();
            let share_before = self.polls[rival_index];
            let counter_swing = self.rng.between(0.02, 0.09);
            self.apply_segment_swing(rival_index, counter_swing, &[]);
            Some(RivalResponse {
                candidate_name,
                share_before,
                share_after: self.polls[rival_index],
            })
        } else {
            None
        };

        let sanction_trust_penalty = if negative_tactic_detected { 16.0 } else { 0.0 };
        let sanction_turnout_penalty = if negative_tactic_detected { 0.16 } else { 0.0 };
        self.trust = (self.trust + action.trust_delta - sanction_trust_penalty).clamp(0.0, 100.0);
        let field_turnout = if is_field_action(id) {
            self.field_capacity * 0.0015
        } else {
            0.0
        };
        self.turnout_momentum = (self.turnout_momentum + action.turnout_delta + field_turnout
            - sanction_turnout_penalty)
            .clamp(0.0, 100.0);
        self.field_capacity = (self.field_capacity + preview.infrastructure_delta).min(60.0);
        if id == ActionId::CampaignHeadquarters {
            self.headquarters_established = true;
        }
        if id == ActionId::SupportersAssociation {
            self.supporters_association_established = true;
        }
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
            self.budget = (self.budget + self.setup.difficulty.weekly_funding()).min(MAX_BUDGET);
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
            id,
            action_name: action.name,
            category: id.category(),
            rival_response,
            event_title,
            share_before,
            share_after: self.polls[self.player_index],
            expected_support_delta: preview.expected_support_delta,
            detection_probability: preview.detection_probability,
            negative_tactic_detected,
            sanction_fine,
            exposed_tactics: self.exposed_tactics,
            sanctioned_funds: self.sanctioned_funds,
            field_capacity_after: self.field_capacity,
            tactic_note,
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

    fn observe_government_action(
        &mut self,
        action: ActionId,
        issue: Issue,
        event_issue: Option<Issue>,
        negative_tactic_detected: bool,
    ) {
        let (approval_delta, issue_delta) = match (self.side, action) {
            (Side::Incumbent, ActionId::PolicyDelivery) => (0.35, 0.65),
            (Side::Incumbent, ActionId::CrisisResponse | ActionId::DisasterResponseVisit) => {
                (0.25, 0.55)
            }
            (Side::Incumbent, ActionId::PublicServices | ActionId::VulnerableCommunityCare) => {
                (0.30, 0.60)
            }
            (Side::Incumbent, ActionId::TransparencyReport | ActionId::FactCheckResponse) => {
                (0.12, 0.30)
            }
            (Side::Challenger, ActionId::OversightPetition) => (-0.12, -0.35),
            (Side::Challenger, ActionId::FactCheckResponse) => (-0.08, -0.20),
            _ => (0.0, 0.0),
        };
        let integrity_penalty = if negative_tactic_detected && self.side == Side::Incumbent {
            -0.55
        } else {
            0.0
        };
        self.government_opinion.observe(
            issue,
            approval_delta + integrity_penalty,
            issue_delta + if negative_tactic_detected { -0.45 } else { 0.0 },
            0.30 + if event_issue == Some(issue) {
                0.45
            } else {
                0.0
            },
        );
        if let Some(event_issue) = event_issue.filter(|event_issue| *event_issue != issue) {
            self.government_opinion.observe(event_issue, 0.0, 0.0, 0.45);
        }
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
        if !self.government_opinion.administration_approval.is_finite()
            || !(0.0..=100.0).contains(&self.government_opinion.administration_approval)
            || self
                .government_opinion
                .issue_approval
                .iter()
                .chain(self.government_opinion.issue_salience.iter())
                .any(|value| !value.is_finite() || !(0.0..=100.0).contains(value))
        {
            return Err("government opinion values must be finite and within 0..=100");
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
        let expected_field_capacity = f64::from(self.headquarters_established) * 35.0
            + f64::from(self.supporters_association_established) * 20.0;
        if !self.field_capacity.is_finite()
            || !(0.0..=55.0).contains(&self.field_capacity)
            || (self.field_capacity - expected_field_capacity).abs() > f64::EPSILON
        {
            return Err("field capacity must match established campaign infrastructure");
        }
        if self.sanctioned_funds < 0 || self.exposed_tactics > CAMPAIGN_ROUNDS * DAYS_PER_WEEK {
            return Err(
                "sanction totals and exposed tactic counts must remain non-negative and bounded",
            );
        }
        let negative_action_count = [
            ActionId::FabricatedPoll,
            ActionId::AnonymousSmear,
            ActionId::CoordinatedOpinionWar,
        ]
        .iter()
        .map(|id| self.action_uses[action_index(*id)])
        .fold(0_u8, u8::saturating_add);
        if self.exposed_tactics > negative_action_count {
            return Err("exposed tactics cannot exceed the number of negative tactics used");
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
        if !self.story_progress.is_valid(self.round) {
            return Err(
                "story choices must follow the campaign schedule and their valid choice sets",
            );
        }
        if !self.staffer_progress.is_valid(self.round) {
            return Err(
                "staffer decisions must follow the campaign schedule and their valid choice sets",
            );
        }
        match self.setup.timeline {
            EventTimeline::SeededWeekly => {
                if (!self.events.is_empty()
                    && (self.event_schedule.len() != CAMPAIGN_ROUNDS as usize
                        || self.event_schedule.iter().any(Option::is_none)))
                    || (self.events.is_empty() && !self.event_schedule.is_empty())
                {
                    return Err("seeded event scheduling must fill each campaign week when events are available");
                }
            }
            EventTimeline::AnniversaryMapped => {
                if self.event_schedule.len() != CAMPAIGN_ROUNDS as usize {
                    return Err(
                        "anniversary event scheduling must provide one slot per campaign week",
                    );
                }
            }
        }
        if self.event_schedule.iter().flatten().any(|event| {
            !self
                .events
                .iter()
                .any(|eligible| eligible.title == event.title)
        }) {
            return Err("scheduled events must come from the selected event pack");
        }
        if self.weekly_objective_schedule.len() != CAMPAIGN_ROUNDS as usize
            || self.weekly_objective_schedule.iter().any(|objective| {
                objective.target_actions == 0
                    || objective.qualifying_actions.is_empty()
                    || objective
                        .qualifying_actions
                        .iter()
                        .any(|id| find_campaign_action(self.side, *id).is_none())
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
        ActionId::PolicyDelivery
        | ActionId::CrisisResponse
        | ActionId::PolicyForum
        | ActionId::Debate
        | ActionId::PolicyPresentation => attributes.policy,
        ActionId::CommunityTour
        | ActionId::PublicServices
        | ActionId::LegalAssembly
        | ActionId::CoalitionTalks
        | ActionId::CampaignHeadquarters
        | ActionId::VolunteerCanvassing
        | ActionId::GroundTeamLeaflets
        | ActionId::StreetSweep
        | ActionId::TurnoutMobilization
        | ActionId::SupportersAssociation
        | ActionId::ReligiousCommunityVisit
        | ActionId::PilgrimageParticipation
        | ActionId::MassAttendance
        | ActionId::VulnerableCommunityCare
        | ActionId::DisasterResponseVisit => attributes.organization,
        ActionId::TransparencyReport
        | ActionId::OversightPetition
        | ActionId::PublicMessage
        | ActionId::RestAndPrepare
        | ActionId::PollCoordination
        | ActionId::MediaInterview
        | ActionId::FactCheckResponse
        | ActionId::FabricatedPoll
        | ActionId::AnonymousSmear
        | ActionId::CoordinatedOpinionWar => attributes.communication,
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

fn make_anniversary_event_schedule(
    events: &[&'static EventCard],
    election_date: &str,
) -> Vec<Option<&'static EventCard>> {
    let mut schedule = vec![None; CAMPAIGN_ROUNDS as usize];
    let mut nearest_date_offsets = vec![None; CAMPAIGN_ROUNDS as usize];
    for event in events {
        let Some(dated_event) = DATED_EVENTS.iter().find(|dated| dated.title == event.title) else {
            continue;
        };
        let Some(days_before) = anniversary_day_offset(dated_event.date, election_date) else {
            continue;
        };
        let week = 11 - (days_before / 7) as u8;
        let slot = usize::from(week);
        if nearest_date_offsets[slot].is_none_or(|previous| days_before < previous) {
            schedule[slot] = Some(*event);
            nearest_date_offsets[slot] = Some(days_before);
        }
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
    id.index()
}

fn is_field_action(id: ActionId) -> bool {
    matches!(
        id,
        ActionId::VolunteerCanvassing
            | ActionId::GroundTeamLeaflets
            | ActionId::StreetSweep
            | ActionId::TurnoutMobilization
    )
}

fn local_slate(region: &RegionProfile) -> ([&'static str; 3], [f64; 3]) {
    match region.previous_winner_party {
        "曙光民主聯盟" => (
            ["曙光民主聯盟", "群島共治黨", "公民新聲黨"],
            [43.0, 37.0, 20.0],
        ),
        "海岳協作黨" => (
            ["海岳協作黨", "曙光民主聯盟", "群島共治黨"],
            [39.0, 33.0, 28.0],
        ),
        "地方共好聯盟" => (
            ["地方共好聯盟", "群島共治黨", "社會同行黨"],
            [38.0, 34.0, 28.0],
        ),
        _ => (
            ["群島共治黨", "曙光民主聯盟", "公民新聲黨"],
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
        assert_eq!(
            game.budget,
            before - 3 + crate::campaign::CampaignDifficulty::Standard.weekly_funding()
        );
        assert!(game.polls.iter().all(|share| *share >= 0.0));
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn action_outcome_records_rival_countermovement_for_campaign_recap() {
        let mut game = GameState::new(2024, Side::Challenger, 707);
        let action = game.apply_action(ActionId::CommunityTour).unwrap();
        let rival = action
            .rival_response
            .expect("a campaigning action should draw a rival response");
        assert_ne!(
            rival.candidate_name,
            game.candidates[game.player_index].name
        );
        assert!(rival.share_after > rival.share_before);

        let rest = game.apply_action(ActionId::RestAndPrepare).unwrap();
        assert!(rest.rival_response.is_none());
    }

    #[test]
    fn action_ids_have_stable_indexes_and_unique_batch_keys() {
        let mut keys = std::collections::HashSet::new();
        for (index, id) in ActionId::ALL.iter().copied().enumerate() {
            assert_eq!(id.index(), index);
            assert!(keys.insert(id.key()));
        }
        assert_eq!(ActionId::ALL.len(), ActionId::COUNT);
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
        assert!(preview.action_description.contains("政策"));
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
            .filter_map(|event| event.map(|event| event.title))
            .collect();
        let right_titles: Vec<_> = right
            .event_schedule
            .iter()
            .filter_map(|event| event.map(|event| event.title))
            .collect();

        assert_eq!(left_titles, right_titles);
        assert_eq!(left.event_schedule.len(), CAMPAIGN_ROUNDS as usize);
        assert!(left_titles.windows(2).all(|pair| pair[0] != pair[1]));
        assert!(left.event_schedule.iter().flatten().all(|event| left
            .events
            .iter()
            .any(|eligible| eligible.title == event.title)));
        assert!(left.validate().is_ok());
    }

    #[test]
    fn anniversary_timeline_maps_known_event_and_event_pack_filters_pool() {
        let setup = CampaignSetup {
            timeline: EventTimeline::AnniversaryMapped,
            ..CampaignSetup::default()
        };
        let mapped = GameState::new_with_setup(
            2004,
            Side::Challenger,
            4242,
            CampaignStyle::Balanced,
            ModelParameters::default(),
            setup,
        );
        assert_eq!(
            mapped.event_schedule[11].map(|event| event.title),
            Some("港區倉儲承攬資料疑義")
        );
        assert_eq!(mapped.event_schedule.len(), CAMPAIGN_ROUNDS as usize);
        assert!(mapped.validate().is_ok());

        let democracy = GameState::new_with_setup(
            2024,
            Side::Challenger,
            4242,
            CampaignStyle::Balanced,
            ModelParameters::default(),
            CampaignSetup {
                event_pack: crate::campaign::EventPack::Democracy,
                timeline: EventTimeline::AnniversaryMapped,
                ..CampaignSetup::default()
            },
        );
        assert!(democracy
            .events
            .iter()
            .all(|event| crate::campaign::EventPack::Democracy.includes(event)));
        assert!(democracy.validate().is_ok());
    }

    #[test]
    fn every_playable_scenario_can_map_at_least_one_fictional_date_anchor() {
        for (year, election_date) in [
            (1996, "1996-10-26"),
            (2000, "2000-10-21"),
            (2004, "2004-10-16"),
            (2008, "2008-10-11"),
            (2012, "2012-10-20"),
            (2016, "2016-10-15"),
            (2020, "2020-10-17"),
            (2024, "2024-10-12"),
            (2026, "2026-10-24"),
        ] {
            let schedule =
                make_anniversary_event_schedule(&events_for_campaign(year), election_date);
            assert!(
                schedule.iter().any(Option::is_some),
                "fictional date mapping has no events for scenario year {year}"
            );
        }
    }

    #[test]
    fn every_fictional_date_anchor_matches_an_event_and_a_playable_campaign_window() {
        for anchor in DATED_EVENTS {
            let event = crate::game_content::EVENTS
                .iter()
                .find(|event| event.title == anchor.title)
                .unwrap_or_else(|| panic!("unmatched date anchor: {}", anchor.title));
            let maps_to_active_cycle = event.active_campaign_years.iter().any(|year| {
                let election_date = if *year == 2026 {
                    Some("2026-10-24")
                } else {
                    crate::game_content::election_for_year(*year).map(|election| election.date)
                };
                election_date
                    .and_then(|date| anniversary_day_offset(anchor.date, date))
                    .is_some()
            });
            assert!(
                maps_to_active_cycle,
                "date anchor does not reach an active cycle: {}",
                anchor.title
            );
        }
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
        let region = crate::game_content::find_region("霧川市").unwrap();
        let game = GameState::new_local_with_style(
            2026,
            Side::Challenger,
            15,
            CampaignStyle::Organizer,
            region,
        );
        assert_eq!(game.region.unwrap().name, "霧川市");
        assert_eq!(game.candidates.len(), 3);
        assert_eq!(game.player_index, 1);
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
    }

    #[test]
    fn multi_candidate_poll_coordination_produces_a_report() {
        let region = crate::game_content::find_region("雲棲市").unwrap();
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
        let region = crate::game_content::find_region("霧川市").unwrap();
        let party = crate::game_content::find_party("山海自治黨").unwrap();
        let mut game = GameState::new_local_with_party(
            2026,
            Side::Challenger,
            888,
            CampaignStyle::Organizer,
            region,
            Some(party),
        );
        assert_eq!(game.candidates[game.player_index].party, "山海自治黨");
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

    #[test]
    fn headquarters_and_supporters_association_build_one_time_field_capacity() {
        let mut game = GameState::new(2024, Side::Challenger, 901);
        let headquarters = game.apply_action(ActionId::CampaignHeadquarters).unwrap();
        assert_eq!(headquarters.field_capacity_after, 35.0);
        assert!(matches!(
            game.apply_action(ActionId::CampaignHeadquarters),
            Err(GameError::HeadquartersAlreadyEstablished)
        ));

        let association = game.apply_action(ActionId::SupportersAssociation).unwrap();
        assert_eq!(association.field_capacity_after, 55.0);
        assert!(game.supporters_association_established);
        assert!(matches!(
            game.apply_action(ActionId::SupportersAssociation),
            Err(GameError::SupportersAssociationAlreadyEstablished)
        ));
        assert!(game.validate().is_ok());
    }

    #[test]
    fn field_capacity_improves_ground_actions_and_religious_events_are_neutral() {
        let baseline = GameState::new(2024, Side::Challenger, 902);
        let base_swing = baseline
            .preview_action(ActionId::StreetSweep)
            .unwrap()
            .expected_swing;
        let mut organized = baseline.clone();
        organized.field_capacity = 55.0;
        let organized_swing = organized
            .preview_action(ActionId::StreetSweep)
            .unwrap()
            .expected_swing;
        assert!((organized_swing - base_swing - 0.1375).abs() < 0.0001);

        for id in [
            ActionId::ReligiousCommunityVisit,
            ActionId::PilgrimageParticipation,
            ActionId::MassAttendance,
        ] {
            assert_eq!(id.category(), TacticCategory::Neutral);
        }
    }

    #[test]
    fn negative_tactics_show_risk_and_exposure_triggers_sanctions() {
        let mut one_attempt = GameState::new(2024, Side::Challenger, 903);
        let initial = one_attempt
            .preview_action(ActionId::FabricatedPoll)
            .unwrap();
        assert_eq!(initial.category, TacticCategory::Negative);
        assert!(initial.detection_probability > 0.0);
        assert!(initial.expected_sanction_fine > 0.0);
        one_attempt.apply_action(ActionId::FabricatedPoll).unwrap();
        let repeated = one_attempt
            .preview_action(ActionId::FabricatedPoll)
            .unwrap();
        assert!(repeated.detection_probability > initial.detection_probability);

        let caught = (0..100)
            .find_map(|seed| {
                let mut game = GameState::new(2024, Side::Challenger, seed);
                let outcome = game.apply_action(ActionId::AnonymousSmear).unwrap();
                outcome.negative_tactic_detected.then_some((game, outcome))
            })
            .expect("a broad deterministic seed range should include an exposed tactic");
        assert_eq!(caught.0.exposed_tactics, 1);
        assert!(caught.1.sanction_fine > 0);
        assert_eq!(caught.0.sanctioned_funds, caught.1.sanction_fine);
        assert!(caught.1.tactic_note.is_some());
        assert!(caught.0.validate().is_ok());
    }

    #[test]
    fn story_choices_resolve_at_scheduled_weeks_without_consuming_a_day() {
        let mut game = GameState::new(2024, Side::Challenger, 904);
        assert!(game.current_story_beat().is_none());
        for _ in 0..DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        let beat = game.current_story_beat().unwrap();
        assert_eq!(beat.week, 2);
        assert!(beat.scene.contains("豪雨"));

        let day_before = game.day;
        let round_before = game.round;
        let outcome = game.apply_story_choice(1).unwrap();
        assert!(outcome.support_delta > 0.0);
        assert_eq!(game.day, day_before);
        assert_eq!(game.round, round_before);
        assert_eq!(game.story_progress.completed_beats(), 1);
        assert!(game.current_story_beat().is_none());
        assert!(game.validate().is_ok());
    }

    #[test]
    fn staffer_choices_are_weekly_non_daily_decisions_with_game_effects() {
        let mut game = GameState::new(2024, Side::Challenger, 717);
        let starting_round = game.round;
        let starting_day = game.day;
        let starting_support = game.polls[game.player_index];
        let first = game.current_staffer_brief().unwrap();
        assert_eq!(first.week, 1);
        let outcome = game.apply_staffer_choice(1).unwrap();
        assert!(outcome.support_delta > 0.0);
        assert_eq!(game.polls[game.player_index], outcome.support_after);
        assert_eq!(game.round, starting_round);
        assert_eq!(game.day, starting_day);
        assert!(game.polls[game.player_index] > starting_support);
        assert!(game.current_staffer_brief().is_none());
        assert!(matches!(
            game.apply_staffer_choice(0),
            Err(GameError::StafferDecisionUnavailable)
        ));
        game.validate().unwrap();

        for _ in 0..(DAYS_PER_WEEK * 2) {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        let third_week = game.current_staffer_brief().unwrap();
        assert_eq!(third_week.week, 3);
        assert!(third_week.scene.contains("誠信") || third_week.scene.contains("資料"));
        game.skip_staffer_brief().unwrap();
        assert!(game.current_staffer_brief().is_none());
        game.validate().unwrap();
    }

    #[test]
    fn staffer_choice_rejects_unaffordable_plans_without_advancing() {
        let mut game = GameState::new(2024, Side::Challenger, 818);
        game.budget = 0;
        let before_support = game.polls[game.player_index];
        assert!(matches!(
            game.apply_staffer_choice(0),
            Err(GameError::InsufficientBudget { .. })
        ));
        assert_eq!(game.round, 0);
        assert_eq!(game.day, 0);
        assert_eq!(game.polls[game.player_index], before_support);
        assert!(game.current_staffer_brief().is_some());
    }

    #[test]
    fn government_opinion_and_candidate_support_are_separate_game_indicators() {
        let mut game = GameState::new(2024, Side::Incumbent, 1201);
        let candidate_polls = game.polls.clone();
        let starting_approval = game.government_opinion.administration_approval;
        game.government_opinion
            .observe(Issue::Education, 1.0, -2.0, 3.0);
        assert_eq!(game.polls, candidate_polls);
        assert_eq!(
            game.government_opinion.administration_approval,
            starting_approval + 1.0
        );
        assert_eq!(game.government_opinion.issue_rating(Issue::Education), 48.0);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn staffer_decision_adds_issue_intelligence_without_confusing_it_with_approval() {
        let mut game = GameState::new(2024, Side::Challenger, 1202);
        let approval_before = game.government_opinion.administration_approval;
        let choice = game.current_staffer_brief().unwrap().choices[0];
        let salience_before = game.government_opinion.issue_priority(choice.issue);
        game.apply_staffer_choice(0).unwrap();
        assert_eq!(
            game.government_opinion.administration_approval,
            approval_before
        );
        assert!(game.government_opinion.issue_priority(choice.issue) > salience_before);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn story_choices_check_budget_and_skips_keep_the_chain_moving() {
        let mut game = GameState::new(2024, Side::Challenger, 905);
        for _ in 0..DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        game.budget = 0;
        let polls_before = game.polls.clone();
        assert!(matches!(
            game.apply_story_choice(0),
            Err(GameError::InsufficientBudget { .. })
        ));
        assert_eq!(game.polls, polls_before);
        assert_eq!(game.story_progress.completed_beats(), 0);
        game.skip_story_beat().unwrap();
        assert_eq!(
            game.story_progress.summary(),
            "skip>pending>pending>pending>pending>pending"
        );
        for _ in 0..2 * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        assert_eq!(game.current_story_beat().unwrap().week, 4);
        game.skip_story_beat().unwrap();
        for _ in 0..2 * DAYS_PER_WEEK {
            game.apply_action(ActionId::RestAndPrepare).unwrap();
        }
        let next = game.current_story_beat().unwrap();
        assert_eq!(next.week, 6);
        assert!(next.scene.contains("前次暫未處理"));
        assert!(game.validate().is_ok());
    }
}
