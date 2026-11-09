//! Citizen-initiated recall campaign simulation.
//!
//! Impeachment is deliberately outside this module. All playable characters,
//! districts, events, and starting conditions are fictional composites.

use std::fmt;

pub const PROPOSAL_DAYS: u16 = 14;
pub const SIGNATURE_DAYS: u16 = 60;
pub const VOTE_CAMPAIGN_DAYS: u16 = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecallSide {
    Initiator,
    Opposition,
}

impl RecallSide {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Initiator => "罷免提議／連署方",
            Self::Opposition => "反對罷免方",
        }
    }

    const fn opponent(self) -> Self {
        match self {
            Self::Initiator => Self::Opposition,
            Self::Opposition => Self::Initiator,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecallTemplate {
    Synthetic,
    LowTurnoutThreshold,
    ConcentratedSupport,
    HighMobilization,
}

impl RecallTemplate {
    pub const ALL: [Self; 4] = [
        Self::Synthetic,
        Self::LowTurnoutThreshold,
        Self::ConcentratedSupport,
        Self::HighMobilization,
    ];

    pub const fn target_name(self) -> &'static str {
        match self {
            Self::Synthetic => "林嶼衡",
            Self::LowTurnoutThreshold => "阮之禾",
            Self::ConcentratedSupport => "卓聞川",
            Self::HighMobilization => "顧以沐",
        }
    }

    pub const fn district_name(self) -> &'static str {
        match self {
            Self::Synthetic => "霧川市海岬區",
            Self::LowTurnoutThreshold => "望洲縣南澗區",
            Self::ConcentratedSupport => "青汀市河岫區",
            Self::HighMobilization => "遙港市秋原區",
        }
    }

    pub const fn target_party(self) -> &'static str {
        match self {
            Self::Synthetic => "群島共治黨",
            Self::LowTurnoutThreshold => "海岳協作黨",
            Self::ConcentratedSupport => "曙光民主聯盟",
            Self::HighMobilization => "公民新聲黨",
        }
    }

    pub const fn constituency_issue(self) -> &'static str {
        match self {
            Self::Synthetic => "港區公車班距與人行動線",
            Self::LowTurnoutThreshold => "農產轉運與雨季接駁",
            Self::ConcentratedSupport => "校園整修排序與公共資訊",
            Self::HighMobilization => "外圍社區醫療接駁與服務回訪",
        }
    }

    pub const fn target_record(self) -> f64 {
        match self {
            Self::Synthetic => 48.0,
            Self::LowTurnoutThreshold => 55.0,
            Self::ConcentratedSupport => 42.0,
            Self::HighMobilization => 61.0,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "synthetic" | "fictional" | "虛構" | "合成" => Some(Self::Synthetic),
            "low-turnout" | "低投票參與" => Some(Self::LowTurnoutThreshold),
            "concentrated" | "集中動員" => Some(Self::ConcentratedSupport),
            "high-mobilization" | "高強度動員" => Some(Self::HighMobilization),
            _ => None,
        }
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Synthetic => "synthetic",
            Self::LowTurnoutThreshold => "low-turnout",
            Self::ConcentratedSupport => "concentrated",
            Self::HighMobilization => "high-mobilization",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Synthetic => "合成選區",
            Self::LowTurnoutThreshold => "低投票參與盤型",
            Self::ConcentratedSupport => "集中支持、低反方動員盤型",
            Self::HighMobilization => "高強度雙方動員盤型",
        }
    }

    pub const fn briefing(self) -> &'static str {
        match self {
            Self::Synthetic => "虛構選區；支持度、組織及出席率均為本局情境參數。",
            Self::LowTurnoutThreshold => {
                "虛構盤型：同意傾向可能領先，但投票參與若不足，仍可能跨不過絕對票數門檻。"
            }
            Self::ConcentratedSupport => {
                "虛構盤型：同意方組織集中、反對方動員不足時，低總投票率也可能越過門檻。"
            }
            Self::HighMobilization => {
                "虛構盤型：雙方支持度與動員程度都高；反罷免方可選擇降溫或積極催票。"
            }
        }
    }

    const fn electorate(self) -> u64 {
        match self {
            Self::Synthetic => 100_000,
            Self::LowTurnoutThreshold => 120_000,
            Self::ConcentratedSupport => 340_000,
            Self::HighMobilization => 1_200_000,
        }
    }

    const fn initial_polls(self) -> (f64, f64, f64, f64) {
        // Synthetic pre-campaign conditions, not historical pre-election polls.
        match self {
            Self::Synthetic => (34.0, 30.0, 55.0, 52.0),
            Self::LowTurnoutThreshold => (37.0, 28.0, 54.0, 57.0),
            Self::ConcentratedSupport => (39.0, 12.0, 68.0, 29.0),
            Self::HighMobilization => (51.0, 8.0, 75.0, 38.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecallPhase {
    Proposal,
    Signature,
    VoteCampaign,
    Complete,
}

impl RecallPhase {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Proposal => "第一階段｜提議",
            Self::Signature => "第二階段｜連署",
            Self::VoteCampaign => "第三階段｜罷免投票宣傳",
            Self::Complete => "本案已結束",
        }
    }

    const fn blocks(self) -> u8 {
        match self {
            Self::Proposal => 2,
            Self::Signature => 6,
            Self::VoteCampaign => 4,
            Self::Complete => 0,
        }
    }

    const fn block_days(self) -> u16 {
        match self {
            Self::Proposal => 7,
            Self::Signature => 10,
            Self::VoteCampaign => 7,
            Self::Complete => 0,
        }
    }

    const fn max_days(self) -> u16 {
        match self {
            Self::Proposal => PROPOSAL_DAYS,
            Self::Signature => SIGNATURE_DAYS,
            Self::VoteCampaign => VOTE_CAMPAIGN_DAYS,
            Self::Complete => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecallActionId {
    BuildCoalition,
    CommunityListening,
    EvidenceBriefing,
    SignatureStations,
    DoorToDoor,
    PublicForum,
    FieldMobilization,
    PublicDefense,
    LocalService,
    CounterMobilization,
    QuietDeescalation,
    LegalAudit,
    FalseSignatures,
    DeceasedSignatures,
}

impl RecallActionId {
    const COUNT: usize = 14;

    const fn index(self) -> usize {
        match self {
            Self::BuildCoalition => 0,
            Self::CommunityListening => 1,
            Self::EvidenceBriefing => 2,
            Self::SignatureStations => 3,
            Self::DoorToDoor => 4,
            Self::PublicForum => 5,
            Self::FieldMobilization => 6,
            Self::PublicDefense => 7,
            Self::LocalService => 8,
            Self::CounterMobilization => 9,
            Self::QuietDeescalation => 10,
            Self::LegalAudit => 11,
            Self::FalseSignatures => 12,
            Self::DeceasedSignatures => 13,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RecallActionSpec {
    pub id: RecallActionId,
    pub label: &'static str,
    pub description: &'static str,
    pub cost: u8,
}

const ACTIONS: &[RecallActionSpec] = &[
    RecallActionSpec {
        id: RecallActionId::BuildCoalition,
        label: "成立公民協作網與地方聯絡點",
        description: "把公民團體、地方志工及支持者分工串接；擴大組織量能，也需要維持合作界線透明。",
        cost: 7,
    },
    RecallActionSpec {
        id: RecallActionId::CommunityListening,
        label: "地方傾聽與逐戶說明",
        description: "整理居民陳情與可查證訴求，回應選區關注；不假定所有接觸都會轉成支持。",
        cost: 4,
    },
    RecallActionSpec {
        id: RecallActionId::EvidenceBriefing,
        label: "公開理由書與查證資料",
        description: "以文件、來源和回應權呈現主張；可信度可能增加，但效果取決於在地議題契合。",
        cost: 5,
    },
    RecallActionSpec {
        id: RecallActionId::SignatureStations,
        label: "設置連署據點與志工排班",
        description: "增加合法連署服務時段與行政核對；有效連署仍以查對結果為準。",
        cost: 6,
    },
    RecallActionSpec {
        id: RecallActionId::DoorToDoor,
        label: "社區拜訪與連署說明",
        description: "逐區安排面對面說明、回收問題並追蹤有效名冊；需要人力與時間。",
        cost: 5,
    },
    RecallActionSpec {
        id: RecallActionId::PublicForum,
        label: "公民論壇與公開辯論",
        description: "讓支持與反對理由同場交鋒；提高議題能見度，也可能帶動對方反動員。",
        cost: 6,
    },
    RecallActionSpec {
        id: RecallActionId::FieldMobilization,
        label: "志工掃街與投票提醒",
        description: "整合地面行程、交通資訊及合法催票，提升支持方出席準備。",
        cost: 7,
    },
    RecallActionSpec {
        id: RecallActionId::PublicDefense,
        label: "提出事實答辯與公開說明",
        description: "逐項回應罷免理由，公布可供檢驗的紀錄並接受追問。",
        cost: 5,
    },
    RecallActionSpec {
        id: RecallActionId::LocalService,
        label: "地方服務紀錄與陳情回訪",
        description: "補上服務成果及未完成事項的進度，將個案回覆和後續責任一併說明。",
        cost: 5,
    },
    RecallActionSpec {
        id: RecallActionId::CounterMobilization,
        label: "組織反對罷免支持者投票",
        description: "建立地方聯絡、分區催票及投票日提醒；只靠聲明不會自動形成不同意票。",
        cost: 7,
    },
    RecallActionSpec {
        id: RecallActionId::QuietDeescalation,
        label: "降低對立、維持低溫回應",
        description: "降低衝突熱度並避免過度升高全國對決；可能減少反感，也有支持者不出門的風險。",
        cost: 2,
    },
    RecallActionSpec {
        id: RecallActionId::LegalAudit,
        label: "依法提出名冊查核與程序意見",
        description: "針對程序及名冊疑義循法定管道處理；不能把合法連署直接當作無效。",
        cost: 4,
    },
    RecallActionSpec {
        id: RecallActionId::FalseSignatures,
        label: "造假連署（高風險違規）",
        description:
            "以不實名冊灌高表面數量；不會計入有效連署，查獲後將觸發名冊刪除、信任崩落與資源懲處。",
        cost: 2,
    },
    RecallActionSpec {
        id: RecallActionId::DeceasedSignatures,
        label: "死亡連署（高風險違規）",
        description:
            "以已死亡者名義灌高表面數量；不會計入有效連署，查獲後將觸發嚴重名冊核查與更重懲處。",
        cost: 2,
    },
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecallError {
    CampaignComplete,
    ActionUnavailable,
    InsufficientBudget { required: u8, available: u8 },
}

impl fmt::Display for RecallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CampaignComplete => f.write_str("本罷免案已結束。"),
            Self::ActionUnavailable => f.write_str("這個階段不能採取該行動。"),
            Self::InsufficientBudget {
                required,
                available,
            } => {
                write!(f, "行動需要預算 {required}，目前剩餘 {available}。")
            }
        }
    }
}

impl std::error::Error for RecallError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecallEnding {
    ProposalFailed,
    SignaturesFailed,
    RecallPassed,
    RecallRejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecallResult {
    pub ending: RecallEnding,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub turnout: u64,
    pub yes_floor: u64,
    pub valid_signatures: u64,
    pub signature_threshold: u64,
    pub reason: &'static str,
}

#[derive(Clone, Debug)]
pub struct RecallState {
    pub template: RecallTemplate,
    pub player_side: RecallSide,
    pub phase: RecallPhase,
    pub electorate: u64,
    pub proposal_threshold: u64,
    pub signature_threshold: u64,
    pub yes_floor: u64,
    pub proposal_count: u64,
    pub genuine_signatures: u64,
    pub fraudulent_signatures: u64,
    pub signature_quality: f64,
    pub yes_intention_pct: f64,
    pub no_intention_pct: f64,
    pub yes_mobilization: f64,
    pub no_mobilization: f64,
    pub initiator_trust: f64,
    pub opposition_trust: f64,
    /// In-game rating of the target's constituency service record.
    pub target_record: f64,
    pub volunteer_capacity: f64,
    pub signature_headwind: f64,
    pub audit_pressure: f64,
    pub fraud_attempts: u8,
    pub fraud_exposed: bool,
    pub budget: u8,
    pub days_elapsed: u16,
    pub blocks_elapsed: u8,
    pub result: Option<RecallResult>,
    seed: u64,
    action_uses: [[u8; RecallActionId::COUNT]; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub struct RecallActionOutcome {
    pub actor: RecallSide,
    pub id: RecallActionId,
    pub label: &'static str,
    pub note: String,
    pub phase_before: RecallPhase,
    pub phase_after: RecallPhase,
    pub days_elapsed: u16,
    pub budget_after: u8,
    pub report: RecallTurnReport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RecallTurnReport {
    pub proposal_before: u64,
    pub proposal_after: u64,
    pub valid_signatures_before: u64,
    pub valid_signatures_after: u64,
    pub yes_intention_before: f64,
    pub yes_intention_after: f64,
    pub no_intention_before: f64,
    pub no_intention_after: f64,
    pub yes_votes_before: u64,
    pub yes_votes_after: u64,
    pub no_votes_before: u64,
    pub no_votes_after: u64,
    pub target_record_before: f64,
    pub target_record_after: f64,
}

impl RecallState {
    pub fn new(template: RecallTemplate, player_side: RecallSide, seed: u64) -> Self {
        let electorate = template.electorate();
        let (yes_intention_pct, no_intention_pct, yes_mobilization, no_mobilization) =
            template.initial_polls();
        Self {
            template,
            player_side,
            phase: RecallPhase::Proposal,
            electorate,
            proposal_threshold: ceil_percent(electorate, 1),
            signature_threshold: ceil_percent(electorate, 10),
            yes_floor: ceil_percent(electorate, 25),
            proposal_count: 0,
            genuine_signatures: 0,
            fraudulent_signatures: 0,
            signature_quality: 0.96,
            yes_intention_pct,
            no_intention_pct,
            yes_mobilization,
            no_mobilization,
            initiator_trust: 50.0,
            opposition_trust: 50.0,
            target_record: template.target_record(),
            volunteer_capacity: 18.0,
            signature_headwind: 0.0,
            audit_pressure: 0.0,
            fraud_attempts: 0,
            fraud_exposed: false,
            budget: 70,
            days_elapsed: 0,
            blocks_elapsed: 0,
            result: None,
            seed: if seed == 0 {
                0x5457_4e52_4543_414c
            } else {
                seed
            },
            action_uses: [[0; RecallActionId::COUNT]; 2],
        }
    }

    pub fn valid_signatures(&self) -> u64 {
        (self.genuine_signatures as f64 * self.signature_quality)
            .floor()
            .max(0.0) as u64
    }

    pub fn raw_signature_count(&self) -> u64 {
        self.genuine_signatures
            .saturating_add(self.fraudulent_signatures)
    }

    pub fn estimated_vote_totals(&self) -> (u64, u64) {
        let yes_turnout_rate = (0.25 + self.yes_mobilization * 0.006).clamp(0.25, 0.85);
        let no_turnout_rate = (0.25 + self.no_mobilization * 0.006).clamp(0.25, 0.85);
        let yes = (self.electorate as f64 * self.yes_intention_pct / 100.0 * yes_turnout_rate)
            .round() as u64;
        let no = (self.electorate as f64 * self.no_intention_pct / 100.0 * no_turnout_rate).round()
            as u64;
        (yes, no)
    }

    pub fn current_block_limit(&self) -> u8 {
        self.phase.blocks()
    }

    pub fn current_days_limit(&self) -> u16 {
        self.phase.max_days()
    }

    pub fn action_options(&self) -> Vec<RecallActionSpec> {
        if self.phase == RecallPhase::Complete {
            return Vec::new();
        }
        ACTIONS
            .iter()
            .copied()
            .filter(|action| self.action_allowed(self.player_side, action.id))
            .collect()
    }

    fn action_allowed(&self, side: RecallSide, action: RecallActionId) -> bool {
        matches!(
            (side, self.phase, action),
            (
                RecallSide::Initiator,
                RecallPhase::Proposal,
                RecallActionId::BuildCoalition
            ) | (
                RecallSide::Initiator,
                RecallPhase::Proposal,
                RecallActionId::CommunityListening
            ) | (
                RecallSide::Initiator,
                RecallPhase::Proposal,
                RecallActionId::EvidenceBriefing
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::BuildCoalition
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::CommunityListening
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::EvidenceBriefing
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::SignatureStations
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::DoorToDoor
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::PublicForum
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::FalseSignatures
            ) | (
                RecallSide::Initiator,
                RecallPhase::Signature,
                RecallActionId::DeceasedSignatures
            ) | (
                RecallSide::Initiator,
                RecallPhase::VoteCampaign,
                RecallActionId::EvidenceBriefing
            ) | (
                RecallSide::Initiator,
                RecallPhase::VoteCampaign,
                RecallActionId::CommunityListening
            ) | (
                RecallSide::Initiator,
                RecallPhase::VoteCampaign,
                RecallActionId::PublicForum
            ) | (
                RecallSide::Initiator,
                RecallPhase::VoteCampaign,
                RecallActionId::FieldMobilization
            ) | (
                RecallSide::Opposition,
                RecallPhase::Proposal,
                RecallActionId::PublicDefense
            ) | (
                RecallSide::Opposition,
                RecallPhase::Proposal,
                RecallActionId::LocalService
            ) | (
                RecallSide::Opposition,
                RecallPhase::Proposal,
                RecallActionId::BuildCoalition
            ) | (
                RecallSide::Opposition,
                RecallPhase::Proposal,
                RecallActionId::CommunityListening
            ) | (
                RecallSide::Opposition,
                RecallPhase::Signature,
                RecallActionId::PublicDefense
            ) | (
                RecallSide::Opposition,
                RecallPhase::Signature,
                RecallActionId::LocalService
            ) | (
                RecallSide::Opposition,
                RecallPhase::Signature,
                RecallActionId::CounterMobilization
            ) | (
                RecallSide::Opposition,
                RecallPhase::Signature,
                RecallActionId::QuietDeescalation
            ) | (
                RecallSide::Opposition,
                RecallPhase::Signature,
                RecallActionId::LegalAudit
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::PublicDefense
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::LocalService
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::CounterMobilization
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::QuietDeescalation
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::LegalAudit
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::PublicForum
            ) | (
                RecallSide::Opposition,
                RecallPhase::VoteCampaign,
                RecallActionId::FieldMobilization
            )
        )
    }

    pub fn apply_action(&mut self, id: RecallActionId) -> Result<RecallActionOutcome, RecallError> {
        if self.phase == RecallPhase::Complete {
            return Err(RecallError::CampaignComplete);
        }
        if !self.action_allowed(self.player_side, id) {
            return Err(RecallError::ActionUnavailable);
        }
        let spec = ACTIONS
            .iter()
            .find(|action| action.id == id)
            .copied()
            .unwrap();
        if self.budget < spec.cost {
            return Err(RecallError::InsufficientBudget {
                required: spec.cost,
                available: self.budget,
            });
        }
        let phase_before = self.phase;
        let (yes_votes_before, no_votes_before) = self.estimated_vote_totals();
        let report_before = RecallTurnReport {
            proposal_before: self.proposal_count,
            proposal_after: self.proposal_count,
            valid_signatures_before: self.valid_signatures(),
            valid_signatures_after: self.valid_signatures(),
            yes_intention_before: self.yes_intention_pct,
            yes_intention_after: self.yes_intention_pct,
            no_intention_before: self.no_intention_pct,
            no_intention_after: self.no_intention_pct,
            yes_votes_before,
            yes_votes_after: yes_votes_before,
            no_votes_before,
            no_votes_after: no_votes_before,
            target_record_before: self.target_record,
            target_record_after: self.target_record,
        };
        self.budget -= spec.cost;
        let note = self.apply_effect(self.player_side, id);
        let opponent_action = self.ai_action(self.player_side.opponent());
        let ai_note = self.apply_effect(self.player_side.opponent(), opponent_action);
        let block_days = self.phase.block_days();
        self.days_elapsed = self.days_elapsed.saturating_add(block_days);
        self.blocks_elapsed = self.blocks_elapsed.saturating_add(1);
        self.budget = self.budget.saturating_add(3).min(100);
        let final_note = format!("{note}；對手回應：{ai_note}");
        self.advance_phase_if_needed();
        let (yes_votes_after, no_votes_after) = self.estimated_vote_totals();
        let report = RecallTurnReport {
            proposal_after: self.proposal_count,
            valid_signatures_after: self.valid_signatures(),
            yes_intention_after: self.yes_intention_pct,
            no_intention_after: self.no_intention_pct,
            yes_votes_after,
            no_votes_after,
            target_record_after: self.target_record,
            ..report_before
        };
        Ok(RecallActionOutcome {
            actor: self.player_side,
            id,
            label: spec.label,
            note: final_note,
            phase_before,
            phase_after: self.phase,
            days_elapsed: self.days_elapsed,
            budget_after: self.budget,
            report,
        })
    }

    fn apply_effect(&mut self, side: RecallSide, action: RecallActionId) -> String {
        let side_index = match side {
            RecallSide::Initiator => 0,
            RecallSide::Opposition => 1,
        };
        let repeated = self.action_uses[side_index][action.index()].min(4);
        self.action_uses[side_index][action.index()] =
            self.action_uses[side_index][action.index()].saturating_add(1);
        let repeat_factor = (1.0 - repeated as f64 * 0.10).clamp(0.55, 1.0);
        match (side, action) {
            (RecallSide::Initiator, RecallActionId::BuildCoalition) => {
                self.volunteer_capacity = (self.volunteer_capacity + 13.0).min(100.0);
                self.initiator_trust = (self.initiator_trust + 2.0).min(100.0);
                if self.phase == RecallPhase::Proposal {
                    self.proposal_count = self.proposal_count.saturating_add(
                        (self.electorate as f64 * 0.0055 * repeat_factor).round() as u64,
                    );
                }
                "地方公民團體與志工分工成形，聯絡網擴大；各團體仍保留自己的主張。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::CommunityListening) => {
                self.increase_yes_intention(0.55 * repeat_factor);
                self.initiator_trust = (self.initiator_trust + 1.4).min(100.0);
                self.volunteer_capacity = (self.volunteer_capacity + 4.0).min(100.0);
                if self.phase == RecallPhase::Proposal {
                    self.proposal_count = self.proposal_count.saturating_add(
                        (self.electorate as f64 * 0.0035 * repeat_factor).round() as u64,
                    );
                }
                "逐區訪談補上在地問題清單；部分居民仍要求提出更具體的問責證據。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::EvidenceBriefing) => {
                self.initiator_trust = (self.initiator_trust + 3.0 * repeat_factor).min(100.0);
                self.increase_yes_intention(0.85 * repeat_factor);
                self.target_record = (self.target_record - 1.4 * repeat_factor).max(0.0);
                self.signature_quality = (self.signature_quality + 0.015).min(0.995);
                if self.phase == RecallPhase::Proposal {
                    self.proposal_count = self.proposal_count.saturating_add(
                        (self.electorate as f64 * 0.0025 * repeat_factor).round() as u64,
                    );
                }
                "理由與來源完成交叉核對，訊息可信度提高；不確定事項仍被清楚標示。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::SignatureStations) => {
                let percent = (1.80 + self.volunteer_capacity * 0.025 - self.signature_headwind)
                    .max(0.2)
                    * repeat_factor;
                self.genuine_signatures = self
                    .genuine_signatures
                    .saturating_add((self.electorate as f64 * percent / 100.0).round() as u64);
                self.volunteer_capacity = (self.volunteer_capacity - 1.0).max(0.0);
                self.initiator_trust = (self.initiator_trust + 0.8).min(100.0);
                "連署站擴增服務時段並核對欄位；有效人數仍須等待選務機關查對。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::DoorToDoor) => {
                let percent = (0.90 + self.volunteer_capacity * 0.018 - self.signature_headwind)
                    .max(0.2)
                    * repeat_factor;
                self.genuine_signatures = self
                    .genuine_signatures
                    .saturating_add((self.electorate as f64 * percent / 100.0).round() as u64);
                self.increase_yes_intention(0.4 * repeat_factor);
                self.volunteer_capacity = (self.volunteer_capacity - 2.0).max(0.0);
                "社區拜訪帶回連署與反對意見，團隊按區域回報有效名冊估計。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::PublicForum) => {
                self.increase_yes_intention(0.9 * repeat_factor);
                self.yes_mobilization = (self.yes_mobilization + 2.0).min(100.0);
                self.initiator_trust = (self.initiator_trust + 1.0).min(100.0);
                "論壇提高議題能見度，也讓反對方取得回應與組織支持者的舞台。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::FieldMobilization) => {
                self.yes_mobilization = (self.yes_mobilization + 9.0 * repeat_factor).min(100.0);
                self.volunteer_capacity = (self.volunteer_capacity + 3.0).min(100.0);
                "志工完成掃街、投票提醒與交通資訊整理，催票名單仍須尊重個人選擇。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::FalseSignatures) => {
                self.fraud_attempts = self.fraud_attempts.saturating_add(1);
                self.fraudulent_signatures = self
                    .fraudulent_signatures
                    .saturating_add((self.electorate as f64 * 0.018).round() as u64);
                self.initiator_trust = (self.initiator_trust - 8.0).max(0.0);
                "表面連署數短暫增加，但不實名冊依法不計入有效連署，且提高查核與處分風險。"
                    .to_string()
            }
            (RecallSide::Initiator, RecallActionId::DeceasedSignatures) => {
                self.fraud_attempts = self.fraud_attempts.saturating_add(1);
                self.fraudulent_signatures = self
                    .fraudulent_signatures
                    .saturating_add((self.electorate as f64 * 0.025).round() as u64);
                self.initiator_trust = (self.initiator_trust - 12.0).max(0.0);
                "名冊顯示異常簽署，表面數量上升但不會成為有效連署；查核壓力大幅提高。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::PublicDefense) => {
                self.yes_intention_pct = (self.yes_intention_pct - 0.65 * repeat_factor).max(0.0);
                self.increase_no_intention(0.8 * repeat_factor);
                self.opposition_trust = (self.opposition_trust + 2.0).min(100.0);
                self.target_record = (self.target_record + 1.2 * repeat_factor).min(100.0);
                self.signature_headwind += 0.10;
                "答辯逐項對照罷免理由與公開紀錄；未能解答的部分成為新的追問。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::LocalService) => {
                self.increase_no_intention(0.45 * repeat_factor);
                self.no_mobilization = (self.no_mobilization + 2.0).min(100.0);
                self.opposition_trust = (self.opposition_trust + 1.6).min(100.0);
                self.target_record = (self.target_record + 2.0 * repeat_factor).min(100.0);
                self.signature_headwind += 0.06;
                "團隊公開地方服務進度並回訪陳情個案，尚未完成的事項列出後續期限。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::CounterMobilization) => {
                self.no_mobilization = (self.no_mobilization + 10.0 * repeat_factor).min(100.0);
                self.increase_no_intention(0.45 * repeat_factor);
                self.volunteer_capacity = (self.volunteer_capacity + 4.0).min(100.0);
                self.signature_headwind += 0.05;
                "反罷免地方聯絡網啟動分區催票；不同意票需由支持者實際出席才能形成。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::QuietDeescalation) => {
                self.yes_mobilization = (self.yes_mobilization - 2.0).max(0.0);
                self.increase_no_intention(0.25);
                self.opposition_trust = (self.opposition_trust + 1.0).min(100.0);
                self.signature_headwind += 0.04;
                "選擇低溫回應，降低全國對決感；部分支持者也因此要求更明確的投票提醒。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::LegalAudit) => {
                self.audit_pressure = (self.audit_pressure + 0.16).min(0.8);
                self.signature_headwind += 0.20;
                "依法提出程序與名冊查核意見，審查仍依選務機關法定程序進行。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::PublicForum) => {
                self.increase_no_intention(0.55 * repeat_factor);
                self.yes_intention_pct = (self.yes_intention_pct - 0.25 * repeat_factor).max(0.0);
                self.no_mobilization = (self.no_mobilization + 3.0).min(100.0);
                "反罷免方在公開論壇提出答辯；提高討論度，也讓支持方更容易催出投票。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::BuildCoalition) => {
                self.volunteer_capacity = (self.volunteer_capacity + 10.0).min(100.0);
                self.no_mobilization = (self.no_mobilization + 3.0).min(100.0);
                self.opposition_trust = (self.opposition_trust + 1.0).min(100.0);
                "地方支持者組成協作網；合作團體的動員意願與政治訊息並不完全相同。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::CommunityListening) => {
                self.increase_no_intention(0.35 * repeat_factor);
                self.opposition_trust = (self.opposition_trust + 1.2).min(100.0);
                "地方行程記錄選民疑慮，答覆是否具體將影響後續信任。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::EvidenceBriefing) => {
                self.increase_no_intention(0.55 * repeat_factor);
                self.yes_intention_pct = (self.yes_intention_pct - 0.2 * repeat_factor).max(0.0);
                self.opposition_trust = (self.opposition_trust + 2.4).min(100.0);
                "整理可查證的政績、問政或程序資料，避免用未證實指控取代答辯。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::FieldMobilization) => {
                self.no_mobilization = (self.no_mobilization + 8.0 * repeat_factor).min(100.0);
                self.volunteer_capacity = (self.volunteer_capacity + 3.0).min(100.0);
                "反罷免志工完成掃街與投票提醒；競選資源轉向投票日動員。".to_string()
            }
            (RecallSide::Opposition, RecallActionId::SignatureStations)
            | (RecallSide::Opposition, RecallActionId::DoorToDoor)
            | (RecallSide::Opposition, RecallActionId::FalseSignatures)
            | (RecallSide::Opposition, RecallActionId::DeceasedSignatures) => {
                "此行動不適用。".to_string()
            }
            (RecallSide::Initiator, RecallActionId::PublicDefense)
            | (RecallSide::Initiator, RecallActionId::LocalService)
            | (RecallSide::Initiator, RecallActionId::CounterMobilization)
            | (RecallSide::Initiator, RecallActionId::QuietDeescalation)
            | (RecallSide::Initiator, RecallActionId::LegalAudit) => "此行動不適用。".to_string(),
        }
    }

    fn ai_action(&mut self, side: RecallSide) -> RecallActionId {
        match side {
            RecallSide::Initiator => match self.phase {
                RecallPhase::Proposal => RecallActionId::BuildCoalition,
                RecallPhase::Signature if self.blocks_elapsed == 0 => {
                    RecallActionId::BuildCoalition
                }
                RecallPhase::Signature => RecallActionId::SignatureStations,
                RecallPhase::VoteCampaign => RecallActionId::FieldMobilization,
                RecallPhase::Complete => RecallActionId::EvidenceBriefing,
            },
            RecallSide::Opposition => match (self.template, self.phase) {
                (_, RecallPhase::Proposal) => RecallActionId::PublicDefense,
                (RecallTemplate::HighMobilization, RecallPhase::Signature) => {
                    RecallActionId::QuietDeescalation
                }
                (_, RecallPhase::Signature) => RecallActionId::LegalAudit,
                (RecallTemplate::LowTurnoutThreshold, RecallPhase::VoteCampaign) => {
                    RecallActionId::PublicForum
                }
                (RecallTemplate::ConcentratedSupport, RecallPhase::VoteCampaign) => {
                    RecallActionId::CounterMobilization
                }
                (RecallTemplate::HighMobilization, RecallPhase::VoteCampaign) => {
                    RecallActionId::CounterMobilization
                }
                (RecallTemplate::Synthetic, RecallPhase::VoteCampaign) => {
                    RecallActionId::FieldMobilization
                }
                (_, RecallPhase::Complete) => RecallActionId::PublicDefense,
            },
        }
    }

    fn advance_phase_if_needed(&mut self) {
        if self.blocks_elapsed < self.phase.blocks() {
            return;
        }
        match self.phase {
            RecallPhase::Proposal => {
                if self.proposal_count < self.proposal_threshold {
                    self.finish_without_vote(
                        RecallEnding::ProposalFailed,
                        "提議人數未達選舉人總數 1% 門檻。",
                    );
                } else {
                    self.phase = RecallPhase::Signature;
                    self.blocks_elapsed = 0;
                }
            }
            RecallPhase::Signature => {
                if self.fraud_attempts > 0 {
                    let exposure_chance =
                        (0.18 + self.fraud_attempts as f64 * 0.24 + self.audit_pressure)
                            .clamp(0.0, 0.95);
                    if self.random_unit() < exposure_chance {
                        self.fraud_exposed = true;
                        self.signature_quality = (self.signature_quality - 0.12).max(0.5);
                        self.initiator_trust = (self.initiator_trust - 28.0).max(0.0);
                        self.budget = self.budget.saturating_sub(18);
                    }
                }
                if self.valid_signatures() < self.signature_threshold {
                    self.finish_without_vote(
                        RecallEnding::SignaturesFailed,
                        "查對後的有效連署未達選舉人總數 10% 門檻。",
                    );
                } else {
                    self.phase = RecallPhase::VoteCampaign;
                    self.blocks_elapsed = 0;
                }
            }
            RecallPhase::VoteCampaign => self.finish_vote(),
            RecallPhase::Complete => {}
        }
    }

    fn finish_without_vote(&mut self, ending: RecallEnding, reason: &'static str) {
        self.phase = RecallPhase::Complete;
        self.result = Some(RecallResult {
            ending,
            yes_votes: 0,
            no_votes: 0,
            turnout: 0,
            yes_floor: self.yes_floor,
            valid_signatures: self.valid_signatures(),
            signature_threshold: self.signature_threshold,
            reason,
        });
    }

    fn finish_vote(&mut self) {
        let (yes_votes, no_votes) = self.estimated_vote_totals();
        let turnout = yes_votes.saturating_add(no_votes);
        let passes = yes_votes >= self.yes_floor && yes_votes > no_votes;
        let reason = if passes {
            "同意票達原選區選舉人總數 1/4，且多於不同意票。"
        } else if yes_votes < self.yes_floor && yes_votes <= no_votes {
            "同意票未達 1/4 門檻，且未多於不同意票。"
        } else if yes_votes < self.yes_floor {
            "同意票多於不同意票，但未達原選區選舉人總數 1/4。"
        } else {
            "同意票達 1/4 門檻，但未多於不同意票。"
        };
        self.phase = RecallPhase::Complete;
        self.result = Some(RecallResult {
            ending: if passes {
                RecallEnding::RecallPassed
            } else {
                RecallEnding::RecallRejected
            },
            yes_votes,
            no_votes,
            turnout,
            yes_floor: self.yes_floor,
            valid_signatures: self.valid_signatures(),
            signature_threshold: self.signature_threshold,
            reason,
        });
    }

    fn random_unit(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.seed >> 11) as f64) / ((1_u64 << 53) as f64)
    }

    fn increase_yes_intention(&mut self, amount: f64) {
        let available = (100.0 - self.no_intention_pct).max(0.0);
        self.yes_intention_pct = (self.yes_intention_pct + amount).min(available);
    }

    fn increase_no_intention(&mut self, amount: f64) {
        let available = (100.0 - self.yes_intention_pct).max(0.0);
        self.no_intention_pct = (self.no_intention_pct + amount).min(available);
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        for value in [
            self.signature_quality,
            self.yes_intention_pct,
            self.no_intention_pct,
            self.yes_mobilization,
            self.no_mobilization,
            self.initiator_trust,
            self.opposition_trust,
            self.target_record,
            self.volunteer_capacity,
        ] {
            if !value.is_finite() || !(0.0..=100.0).contains(&value) {
                return Err("recall state percentages and resources must be within 0..=100");
            }
        }
        if self.proposal_threshold != ceil_percent(self.electorate, 1)
            || self.signature_threshold != ceil_percent(self.electorate, 10)
            || self.yes_floor != ceil_percent(self.electorate, 25)
        {
            return Err("recall statutory thresholds do not match the electorate size");
        }
        if self.yes_intention_pct + self.no_intention_pct > 100.0 + f64::EPSILON {
            return Err("yes and no intention cannot exceed the electorate together");
        }
        if self.phase == RecallPhase::Complete && self.result.is_none() {
            return Err("a completed recall campaign must have a result");
        }
        Ok(())
    }
}

fn ceil_percent(total: u64, percent: u64) -> u64 {
    total.saturating_mul(percent).saturating_add(99) / 100
}

pub fn actions_for_phase(state: &RecallState) -> Vec<RecallActionSpec> {
    state.action_options()
}

pub fn action_from_number(state: &RecallState, selection: usize) -> Option<RecallActionId> {
    state
        .action_options()
        .get(selection.checked_sub(1)?)
        .map(|action| action.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reach_signature_phase(state: &mut RecallState) {
        state.proposal_count = state.proposal_threshold;
        state.blocks_elapsed = RecallPhase::Proposal.blocks();
        state.days_elapsed = PROPOSAL_DAYS;
        state.advance_phase_if_needed();
    }

    #[test]
    fn statutory_thresholds_use_ceiling_and_cases_have_distinct_electorates() {
        let low = RecallState::new(
            RecallTemplate::LowTurnoutThreshold,
            RecallSide::Initiator,
            1,
        );
        let concentrated = RecallState::new(
            RecallTemplate::ConcentratedSupport,
            RecallSide::Initiator,
            1,
        );
        let high = RecallState::new(RecallTemplate::HighMobilization, RecallSide::Initiator, 1);
        assert_eq!(low.proposal_threshold, 1_200);
        assert_eq!(low.signature_threshold, 12_000);
        assert_eq!(low.yes_floor, 30_000);
        assert_eq!(concentrated.signature_threshold, 34_000);
        assert_eq!(high.yes_floor, 300_000);
        assert_ne!(low.electorate, concentrated.electorate);
    }

    #[test]
    fn recall_scenarios_use_distinct_fictional_targets_and_districts() {
        let templates = RecallTemplate::ALL;
        assert_eq!(templates.len(), 4);
        let names = templates.map(RecallTemplate::target_name);
        let districts = templates.map(RecallTemplate::district_name);
        let parties = templates.map(RecallTemplate::target_party);
        assert!(names.iter().all(|name| !name.trim().is_empty()));
        assert!(districts.iter().all(|district| !district.trim().is_empty()));
        for index in 0..templates.len() {
            for other in index + 1..templates.len() {
                assert_ne!(names[index], names[other]);
                assert_ne!(districts[index], districts[other]);
                assert_ne!(parties[index], parties[other]);
            }
        }
    }

    #[test]
    fn recall_target_profile_changes_with_evidence_and_public_defense() {
        let mut initiator = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 100);
        let baseline = initiator.target_record;
        initiator
            .apply_action(RecallActionId::EvidenceBriefing)
            .unwrap();
        assert!(initiator.target_record < baseline);

        let mut opposition =
            RecallState::new(RecallTemplate::Synthetic, RecallSide::Opposition, 100);
        let baseline = opposition.target_record;
        opposition
            .apply_action(RecallActionId::PublicDefense)
            .unwrap();
        assert!(opposition.target_record > baseline);
        assert!(opposition.validate().is_ok());
    }

    #[test]
    fn recall_action_report_exposes_combined_player_and_opponent_response() {
        let mut state = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 103);
        let outcome = state
            .apply_action(RecallActionId::EvidenceBriefing)
            .unwrap();

        assert_eq!(outcome.phase_before, RecallPhase::Proposal);
        assert!(outcome.report.proposal_after > outcome.report.proposal_before);
        assert!(outcome.report.yes_intention_after > outcome.report.yes_intention_before);
        assert!(outcome.report.no_intention_after > outcome.report.no_intention_before);
        assert!(outcome.report.target_record_after < outcome.report.target_record_before);
        assert_eq!(outcome.report.valid_signatures_before, 0);
        assert_eq!(outcome.report.valid_signatures_after, 0);
    }

    #[test]
    fn phases_enforce_14_60_and_28_day_windows_and_fail_before_ballot() {
        let mut state = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 7);
        for _ in 0..2 {
            state
                .apply_action(RecallActionId::EvidenceBriefing)
                .unwrap();
        }
        assert_eq!(state.days_elapsed, PROPOSAL_DAYS);
        assert_eq!(state.phase, RecallPhase::Complete);
        assert_eq!(state.result.unwrap().ending, RecallEnding::ProposalFailed);

        let mut state = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 8);
        reach_signature_phase(&mut state);
        for _ in 0..6 {
            state.genuine_signatures = state.signature_threshold * 2;
            state
                .apply_action(RecallActionId::SignatureStations)
                .unwrap();
        }
        assert_eq!(state.days_elapsed, PROPOSAL_DAYS + SIGNATURE_DAYS);
        assert_eq!(state.phase, RecallPhase::VoteCampaign);
    }

    #[test]
    fn forged_and_deceased_signatures_never_count_as_valid_and_trigger_review_risk() {
        let mut state = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 12);
        reach_signature_phase(&mut state);
        let before = state.initiator_trust;
        state
            .apply_action(RecallActionId::DeceasedSignatures)
            .unwrap();
        assert!(state.raw_signature_count() > 0);
        assert_eq!(state.valid_signatures(), 0);
        assert!(state.initiator_trust < before);
        assert_eq!(state.fraud_attempts, 1);
    }

    #[test]
    fn anti_recall_mobilization_can_defeat_a_yes_floor_that_would_otherwise_pass() {
        let mut state = RecallState::new(
            RecallTemplate::LowTurnoutThreshold,
            RecallSide::Opposition,
            99,
        );
        state.phase = RecallPhase::VoteCampaign;
        state.blocks_elapsed = 0;
        state.yes_intention_pct = 40.0;
        state.no_intention_pct = 42.0;
        state.yes_mobilization = 58.0;
        state.no_mobilization = 50.0;
        for _ in 0..4 {
            state
                .apply_action(RecallActionId::CounterMobilization)
                .unwrap();
        }
        let result = state.result.unwrap();
        assert_eq!(result.ending, RecallEnding::RecallRejected);
        assert!(result.no_votes >= result.yes_votes);
    }

    #[test]
    fn seeded_case_is_repeatable_and_state_validation_is_enforced() {
        let run = |seed| {
            let mut state = RecallState::new(
                RecallTemplate::ConcentratedSupport,
                RecallSide::Initiator,
                seed,
            );
            reach_signature_phase(&mut state);
            for _ in 0..6 {
                state
                    .apply_action(RecallActionId::SignatureStations)
                    .unwrap();
            }
            if state.phase == RecallPhase::VoteCampaign {
                for _ in 0..4 {
                    state
                        .apply_action(RecallActionId::FieldMobilization)
                        .unwrap();
                }
            }
            state.result.unwrap()
        };
        assert_eq!(run(55), run(55));

        let mut state = RecallState::new(RecallTemplate::Synthetic, RecallSide::Initiator, 1);
        state.yes_mobilization = 101.0;
        assert!(state.validate().is_err());
    }
}
