//! Choice-driven campaign story content.
//!
//! Story arcs are fictional composite scenarios. They use recurring characters
//! and consequences as game material; they do not depict a single historical
//! event or claim a fixed political effect.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryChoiceId {
    FieldVisit,
    PublishFacts,
    CoordinateNetwork,
    TrackProgress,
    PublicRoundtable,
    JointOversight,
    VerifiedDeadline,
    ResidentReinspection,
    PublishOpenItems,
    PublishFullClip,
    CandidateInterview,
    IgnoreClip,
    PublishTimeline,
    OpenQuestions,
    SpokespersonBriefing,
    DebateWithRecord,
    InterviewAfterDebate,
    PivotToTurnout,
}

impl StoryChoiceId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::FieldVisit => "field_visit",
            Self::PublishFacts => "publish_facts",
            Self::CoordinateNetwork => "coordinate_network",
            Self::TrackProgress => "track_progress",
            Self::PublicRoundtable => "public_roundtable",
            Self::JointOversight => "joint_oversight",
            Self::VerifiedDeadline => "verified_deadline",
            Self::ResidentReinspection => "resident_reinspection",
            Self::PublishOpenItems => "publish_open_items",
            Self::PublishFullClip => "publish_full_clip",
            Self::CandidateInterview => "candidate_interview",
            Self::IgnoreClip => "ignore_clip",
            Self::PublishTimeline => "publish_timeline",
            Self::OpenQuestions => "open_questions",
            Self::SpokespersonBriefing => "spokesperson_briefing",
            Self::DebateWithRecord => "debate_with_record",
            Self::InterviewAfterDebate => "interview_after_debate",
            Self::PivotToTurnout => "pivot_to_turnout",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryDecision {
    Chosen(StoryChoiceId),
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryThreadStage {
    NotStarted,
    Open,
    FollowedUp,
    Resolved,
    Deferred,
}

impl StoryThreadStage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotStarted => "尚未展開",
            Self::Open => "待回應",
            Self::FollowedUp => "追蹤中",
            Self::Resolved => "已收束",
            Self::Deferred => "暫緩／仍待交代",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryMemory {
    pub resident_trust: u8,
    pub team_trust: u8,
    pub reporter_trust: u8,
    pub rain_thread: StoryThreadStage,
    pub information_thread: StoryThreadStage,
}

impl Default for StoryMemory {
    fn default() -> Self {
        Self {
            resident_trust: 50,
            team_trust: 50,
            reporter_trust: 50,
            rain_thread: StoryThreadStage::NotStarted,
            information_thread: StoryThreadStage::NotStarted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryProgress {
    decisions: [Option<StoryDecision>; 6],
    memory: StoryMemory,
}

impl Default for StoryProgress {
    fn default() -> Self {
        Self {
            decisions: [None; 6],
            memory: StoryMemory::default(),
        }
    }
}

impl StoryProgress {
    pub fn decision(self, beat_index: usize) -> Option<StoryDecision> {
        self.decisions.get(beat_index).copied().flatten()
    }

    pub fn summary(self) -> String {
        self.decisions
            .iter()
            .map(|decision| match decision {
                Some(StoryDecision::Chosen(id)) => id.key(),
                Some(StoryDecision::Skipped) => "skip",
                None => "pending",
            })
            .collect::<Vec<_>>()
            .join(">")
    }

    pub fn completed_beats(self) -> u8 {
        self.decisions
            .iter()
            .filter(|decision| decision.is_some())
            .count() as u8
    }

    pub fn memory(self) -> StoryMemory {
        self.memory
    }

    pub fn memory_summary(self) -> String {
        format!(
            "角色關係：陳秀惠（里民）{}/100、林予安（幕僚）{}/100、周以晴（記者）{}/100；雨後陳情線：{}；公開影像線：{}",
            self.memory.resident_trust,
            self.memory.team_trust,
            self.memory.reporter_trust,
            self.memory.rain_thread.label(),
            self.memory.information_thread.label()
        )
    }

    pub(crate) fn record(&mut self, beat_index: usize, decision: StoryDecision) -> bool {
        let Some(slot) = self.decisions.get_mut(beat_index) else {
            return false;
        };
        if slot.is_some() {
            return false;
        }
        *slot = Some(decision);
        self.memory.remember(beat_index, decision);
        true
    }

    pub(crate) fn is_valid(self, round: u8) -> bool {
        let trigger_rounds = [1_u8, 3, 5, 7, 9, 11];
        for (index, decision) in self.decisions.iter().enumerate() {
            if decision.is_some() && round < trigger_rounds[index] {
                return false;
            }
            if decision.is_some() && index > 0 && self.decisions[index - 1].is_none() {
                return false;
            }
            if let Some(StoryDecision::Chosen(id)) = decision {
                let Some(choices) = choices_for(index) else {
                    return false;
                };
                if !choices.iter().any(|choice| choice.id == *id) {
                    return false;
                }
            }
        }
        true
    }
}

impl StoryMemory {
    fn remember(&mut self, beat_index: usize, decision: StoryDecision) {
        match beat_index {
            0 => {
                self.rain_thread = opening_thread(decision);
                self.resident_trust = adjust_relation(
                    self.resident_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::FieldVisit) => 6,
                        StoryDecision::Chosen(StoryChoiceId::PublishFacts) => 4,
                        StoryDecision::Chosen(StoryChoiceId::CoordinateNetwork) => 2,
                        _ => -3,
                    },
                );
            }
            1 => {
                self.information_thread = opening_thread(decision);
                self.team_trust = adjust_relation(
                    self.team_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::PublishFullClip) => 5,
                        StoryDecision::Chosen(StoryChoiceId::CandidateInterview) => 3,
                        StoryDecision::Chosen(StoryChoiceId::IgnoreClip) => -5,
                        _ => -3,
                    },
                );
            }
            2 => {
                self.rain_thread = followup_thread(self.rain_thread, decision);
                self.resident_trust = adjust_relation(
                    self.resident_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::TrackProgress) => 3,
                        StoryDecision::Chosen(StoryChoiceId::PublicRoundtable) => 4,
                        StoryDecision::Chosen(StoryChoiceId::JointOversight) => 2,
                        _ => -2,
                    },
                );
                self.reporter_trust = adjust_relation(
                    self.reporter_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::TrackProgress) => 5,
                        StoryDecision::Chosen(StoryChoiceId::PublicRoundtable) => 4,
                        StoryDecision::Chosen(StoryChoiceId::JointOversight) => 3,
                        _ => -3,
                    },
                );
            }
            3 => {
                self.information_thread = followup_thread(self.information_thread, decision);
                self.reporter_trust = adjust_relation(
                    self.reporter_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::PublishTimeline) => 6,
                        StoryDecision::Chosen(StoryChoiceId::OpenQuestions) => 5,
                        StoryDecision::Chosen(StoryChoiceId::SpokespersonBriefing) => 2,
                        _ => -3,
                    },
                );
            }
            4 => {
                self.rain_thread = closing_thread(self.rain_thread, decision);
                self.resident_trust = adjust_relation(
                    self.resident_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::VerifiedDeadline) => 4,
                        StoryDecision::Chosen(StoryChoiceId::ResidentReinspection) => 6,
                        StoryDecision::Chosen(StoryChoiceId::PublishOpenItems) => 5,
                        _ => -3,
                    },
                );
            }
            5 => {
                self.information_thread = closing_thread(self.information_thread, decision);
                self.reporter_trust = adjust_relation(
                    self.reporter_trust,
                    match decision {
                        StoryDecision::Chosen(StoryChoiceId::DebateWithRecord) => 5,
                        StoryDecision::Chosen(StoryChoiceId::InterviewAfterDebate) => 4,
                        StoryDecision::Chosen(StoryChoiceId::PivotToTurnout) => 2,
                        _ => -3,
                    },
                );
            }
            _ => {}
        }
    }
}

fn adjust_relation(value: u8, delta: i16) -> u8 {
    (i16::from(value) + delta).clamp(0, 100) as u8
}

fn opening_thread(decision: StoryDecision) -> StoryThreadStage {
    match decision {
        StoryDecision::Chosen(_) => StoryThreadStage::Open,
        StoryDecision::Skipped => StoryThreadStage::Deferred,
    }
}

fn followup_thread(current: StoryThreadStage, decision: StoryDecision) -> StoryThreadStage {
    match decision {
        StoryDecision::Chosen(_) => StoryThreadStage::FollowedUp,
        StoryDecision::Skipped if current == StoryThreadStage::Open => StoryThreadStage::Deferred,
        StoryDecision::Skipped => current,
    }
}

fn closing_thread(current: StoryThreadStage, decision: StoryDecision) -> StoryThreadStage {
    match decision {
        StoryDecision::Chosen(_) => StoryThreadStage::Resolved,
        StoryDecision::Skipped
            if matches!(
                current,
                StoryThreadStage::Open | StoryThreadStage::FollowedUp
            ) =>
        {
            StoryThreadStage::Deferred
        }
        StoryDecision::Skipped => current,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoryChoiceSpec {
    pub id: StoryChoiceId,
    pub label: &'static str,
    pub description: &'static str,
    pub budget_cost: i32,
    pub support_delta: f64,
    pub trust_delta: f64,
    pub mobilization_delta: f64,
    pub stamina_delta: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct StoryBeat {
    pub index: usize,
    pub week: u8,
    pub title: &'static str,
    pub character: &'static str,
    pub scene: &'static str,
    pub choices: &'static [StoryChoiceSpec],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StoryBonus {
    pub support_delta: f64,
    pub trust_delta: f64,
    pub note: Option<&'static str>,
}

const OPENING_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::FieldVisit,
        label: "安排現勘，整理居民通報清單",
        description: "派人確認積水路段與通勤影響；較花體力和經費，但能建立後續回訪基礎。",
        budget_cost: 2,
        support_delta: 0.08,
        trust_delta: 1.5,
        mobilization_delta: 0.8,
        stamina_delta: -3.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::PublishFacts,
        label: "先公開已確認資訊與待查事項",
        description: "先說明已知與未知；降低誤傳風險，但暫時不承諾工程時程。",
        budget_cost: 1,
        support_delta: 0.04,
        trust_delta: 2.0,
        mobilization_delta: 0.1,
        stamina_delta: -1.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::CoordinateNetwork,
        label: "先拜會地方窗口，確認權責分工",
        description: "先找出受理單位與聯絡窗口；可以串起地方網絡，但直接回覆較慢。",
        budget_cost: 1,
        support_delta: 0.06,
        trust_delta: 1.0,
        mobilization_delta: 1.0,
        stamina_delta: -2.0,
    },
];

const FOLLOWUP_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::TrackProgress,
        label: "每週公布處理進度與未完成項目",
        description: "維持透明追蹤；需要持續整理資料，也方便居民核對承諾。",
        budget_cost: 1,
        support_delta: 0.07,
        trust_delta: 1.2,
        mobilization_delta: 0.3,
        stamina_delta: -1.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::PublicRoundtable,
        label: "召開居民、專業者與主管機關協調會",
        description: "讓不同角色對照需求與權責；花費較高，但有機會形成共同行動。",
        budget_cost: 2,
        support_delta: 0.08,
        trust_delta: 1.0,
        mobilization_delta: 0.7,
        stamina_delta: -2.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::JointOversight,
        label: "把未解事項列入跨黨監督清單",
        description: "把仍待處理的部分公開列管；能擴大監督參與，但協調成本較高。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 1.3,
        mobilization_delta: 0.8,
        stamina_delta: -1.0,
    },
];

const CLOSING_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::VerifiedDeadline,
        label: "提出可查核的改善期限與指標",
        description: "把期限和成果指標講清楚；若前面選擇公開進度，能形成一致路線。",
        budget_cost: 2,
        support_delta: 0.08,
        trust_delta: 1.6,
        mobilization_delta: 0.2,
        stamina_delta: -2.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::ResidentReinspection,
        label: "邀請居民代表共同復勘",
        description: "讓受影響居民檢視改善情形；若前面投入現勘與協調，較能接續成果。",
        budget_cost: 2,
        support_delta: 0.09,
        trust_delta: 1.2,
        mobilization_delta: 0.8,
        stamina_delta: -3.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::PublishOpenItems,
        label: "公開仍待處理事項，不作超出權限的保證",
        description: "坦承尚未完成的部分；直接動員較少，但能維持資訊一致。",
        budget_cost: 0,
        support_delta: 0.03,
        trust_delta: 2.0,
        mobilization_delta: 0.0,
        stamina_delta: -1.0,
    },
];

const CLIP_OPENING_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::PublishFullClip,
        label: "公開完整影像、逐字稿與拍攝脈絡",
        description: "一次提供可核對材料；整理成本較高，但讓外界自行比對前後文。",
        budget_cost: 2,
        support_delta: 0.06,
        trust_delta: 1.8,
        mobilization_delta: 0.2,
        stamina_delta: -1.5,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::CandidateInterview,
        label: "由候選人接受即時專訪說明",
        description: "直接回答追問、掌握發言節奏；能即時釐清疑問，但需要更多體力面對追問。",
        budget_cost: 1,
        support_delta: 0.10,
        trust_delta: 0.8,
        mobilization_delta: 0.1,
        stamina_delta: -3.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::IgnoreClip,
        label: "暫不回應，先把行程留給地方組織",
        description: "把資源留在地面動員；議題熱度可能自行下降，也可能留下未回覆的空白。",
        budget_cost: 0,
        support_delta: 0.02,
        trust_delta: -0.8,
        mobilization_delta: 1.0,
        stamina_delta: -0.5,
    },
];

const CLIP_FOLLOWUP_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::PublishTimeline,
        label: "公布時間軸、原始來源與查證進度",
        description: "把可確認的來源、時間與待查點分開列出；透明度提高，但也會留下可供檢驗的紀錄。",
        budget_cost: 2,
        support_delta: 0.07,
        trust_delta: 1.8,
        mobilization_delta: 0.0,
        stamina_delta: -1.5,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::OpenQuestions,
        label: "安排公開問答，接受不同立場追問",
        description: "讓記者與支持者都能提問；對話有機會解除疑慮，也會消耗候選人體力。",
        budget_cost: 1,
        support_delta: 0.08,
        trust_delta: 1.0,
        mobilization_delta: 0.2,
        stamina_delta: -3.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::SpokespersonBriefing,
        label: "由發言人統一說明並更新查核結果",
        description: "降低候選人行程負擔，維持單一資訊窗口；若說明不完整，仍可能被追問。",
        budget_cost: 1,
        support_delta: 0.04,
        trust_delta: 0.6,
        mobilization_delta: 0.4,
        stamina_delta: -0.5,
    },
];

const CLIP_CLOSING_CHOICES: &[StoryChoiceSpec] = &[
    StoryChoiceSpec {
        id: StoryChoiceId::DebateWithRecord,
        label: "帶著完整紀錄在辯論會正面回應",
        description: "把來源與前後文帶上公開舞臺；若先前已公開查證脈絡，回應更連貫。",
        budget_cost: 2,
        support_delta: 0.09,
        trust_delta: 1.5,
        mobilization_delta: 0.2,
        stamina_delta: -2.5,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::InterviewAfterDebate,
        label: "辯論後再接受專訪，逐題補充說明",
        description: "用較長篇幅處理仍有疑問的部分；溝通時間增加，也會壓縮最後衝刺安排。",
        budget_cost: 1,
        support_delta: 0.07,
        trust_delta: 1.2,
        mobilization_delta: 0.1,
        stamina_delta: -2.0,
    },
    StoryChoiceSpec {
        id: StoryChoiceId::PivotToTurnout,
        label: "停止延長攻防，把資源轉回催票",
        description: "尊重已完成的說明、集中最後動員；保留部分選民仍待自行判斷的空間。",
        budget_cost: 0,
        support_delta: 0.03,
        trust_delta: 0.2,
        mobilization_delta: 1.0,
        stamina_delta: -0.5,
    },
];

pub fn beat_for_round(round: u8, progress: StoryProgress) -> Option<StoryBeat> {
    let (index, week, title, character, scene, choices) = match round {
        1 => (
            0,
            2,
            "雨後一條路：陳情、查證與回覆",
            "陳秀惠（虛構里民代表）",
            "一場豪雨後，幾個路口積水，公車也得臨時改道。里民代表陳秀惠帶來通報清單；競選幕僚林予安請團隊決定先從哪裡著手。",
            OPENING_CHOICES,
        ),
        3 => (
            1,
            4,
            "一句話之後：剪輯、查證與信任",
            "林予安（虛構競選幕僚）",
            "一段候選人在公開論壇的發言被剪成短片，片段開始在社群流傳；原始錄影仍在主辦單位手上，團隊收到多方詢問。候選人必須決定先拿出什麼、又要如何面對追問。",
            CLIP_OPENING_CHOICES,
        ),
        5 => (
            2,
            6,
            "雨後一條路：陳情、查證與回覆",
            "周以晴（虛構地方記者）",
            match progress.decision(0) {
                Some(StoryDecision::Chosen(StoryChoiceId::FieldVisit)) => {
                    "前次現勘已確認幾處積水點，但居民仍在追問後續。地方記者周以晴來訪，要求說明哪些項目已處理、哪些還待查。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::PublishFacts)) => {
                    "已公開的資料減少了傳言，居民又補上新的通報。地方記者周以晴追問資訊更新頻率與責任窗口。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::CoordinateNetwork)) => {
                    "地方窗口已找到受影響住戶，但各單位回覆時間不一致。地方記者周以晴要求說明協調進度。"
                }
                Some(StoryDecision::Skipped) => {
                    "這條陳情線前次暫未處理。地方記者周以晴取得新的通報，詢問團隊是否願意公開跟進。"
                }
                _ => "新的通報送到競選團隊。地方記者周以晴追問目前掌握的資訊與下一步安排。",
            },
            FOLLOWUP_CHOICES,
        ),
        7 => (
            3,
            8,
            "一句話之後：剪輯、查證與信任",
            "周以晴（虛構地方記者）",
            match progress.decision(1) {
                Some(StoryDecision::Chosen(StoryChoiceId::PublishFullClip)) => {
                    "完整影像與逐字稿已公開，但不同媒體對關鍵段落仍有不同解讀。周以晴帶著時間碼回到總部，追問團隊能否交代查證流程。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::CandidateInterview)) => {
                    "候選人接受專訪後，部分疑問獲得回答，另一些片段又被拆成新的短影音。周以晴希望團隊說明哪些資訊可以由原始紀錄核對。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::IgnoreClip)) => {
                    "短片熱度稍退，然而地方座談仍有人提起。周以晴詢問候選人：這是已經說明過的議題，還是團隊決定不再回應？"
                }
                Some(StoryDecision::Skipped) => {
                    "團隊前次沒有選擇回應方式，片段仍在流傳。周以晴帶來新的查證問題，候選人需要決定是否建立公開說明的節奏。"
                }
                _ => "原始影像仍待確認，周以晴帶著新的時間碼與問題來訪；團隊必須決定如何讓外界追蹤查證進度。",
            },
            CLIP_FOLLOWUP_CHOICES,
        ),
        9 => (
            4,
            10,
            "雨後一條路：陳情、查證與回覆",
            "陳秀惠（虛構里民代表）",
            match progress.decision(2) {
                Some(StoryDecision::Chosen(StoryChoiceId::TrackProgress)) => {
                    "公開追蹤表已有幾項進度，仍有工程尚未完成。投票日前，陳秀惠希望團隊把期限和未解事項交代清楚。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::PublicRoundtable)) => {
                    "協調會確認了各單位權責，但改善工程還沒全部完成。陳秀惠詢問如何讓居民參與後續檢視。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::JointOversight)) => {
                    "跨黨監督清單列出仍待處理的項目。陳秀惠希望團隊別只公布爭點，也說明接下來的追蹤方式。"
                }
                Some(StoryDecision::Skipped) => {
                    "工程改善尚未全部完成。陳秀惠再次提出陳情，希望團隊在選前說明下一步。"
                }
                _ => "改善工程尚未全部完成。陳秀惠再次提出陳情，希望團隊在選前說明下一步。",
            },
            CLOSING_CHOICES,
        ),
        11 => (
            5,
            12,
            "一句話之後：剪輯、查證與信任",
            "陳秀惠（虛構里民代表）",
            match progress.decision(3) {
                Some(StoryDecision::Chosen(StoryChoiceId::PublishTimeline)) => {
                    "查證時間軸已更新，仍有少數來源等待回覆。投票日前最後一場辯論將再次談到這段影片，陳秀惠提醒團隊：公開紀錄也包括尚未確認的空白。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::OpenQuestions)) => {
                    "公開問答留下逐題紀錄，現場有人認同、也有人仍有疑問。陳秀惠問候選人，最後一次回應要留在辯論會還是交給後續專訪。"
                }
                Some(StoryDecision::Chosen(StoryChoiceId::SpokespersonBriefing)) => {
                    "發言人已整理說明稿，記者仍希望直接問候選人。陳秀惠提醒團隊，選民最後記住的可能不是聲明，而是候選人如何回答追問。"
                }
                Some(StoryDecision::Skipped) => {
                    "團隊沒有建立固定的說明方式，辯論會前仍有選民詢問片段脈絡。陳秀惠希望候選人決定最後一週如何處理這些未解問題。"
                }
                _ => "辯論會前，片段的來源與脈絡仍是媒體提問焦點。陳秀惠希望候選人選擇一種能承擔後果的收尾方式。",
            },
            CLIP_CLOSING_CHOICES,
        ),
        _ => return None,
    };

    if progress.decision(index).is_some() {
        return None;
    }
    Some(StoryBeat {
        index,
        week,
        title,
        character,
        scene,
        choices,
    })
}

/// A delayed callback triggered when a returning character brings up the team's
/// earlier promise or information-handling decision.
pub fn character_memory_echo(week: u8, progress: StoryProgress) -> Option<&'static str> {
    let memory = progress.memory();
    match week {
        6 => match (memory.rain_thread, memory.resident_trust) {
            (StoryThreadStage::Open, trust) if trust >= 55 => {
                Some("陳秀惠記得團隊答應現勘，帶來新的通報，也願意先一起核對處理進度。")
            }
            (StoryThreadStage::Open, _) => {
                Some("陳秀惠記得團隊承諾的後續；她先問上次列出的問題，是否已有能核對的答覆。")
            }
            (StoryThreadStage::Deferred, _) => {
                Some("陳秀惠記得上次團隊沒有承諾處理方式，這次先要求說清楚誰會接手。")
            }
            _ => None,
        },
        10 => match (memory.rain_thread, memory.resident_trust) {
            (StoryThreadStage::FollowedUp, trust) if trust >= 60 => {
                Some("陳秀惠帶著居民共同整理的追蹤表回來，要求把最後期限和責任窗口寫清楚。")
            }
            (StoryThreadStage::FollowedUp, _) => {
                Some("陳秀惠確認過部分追蹤資料，但仍要團隊交代尚未完成的項目。")
            }
            (StoryThreadStage::Deferred, _) => {
                Some("陳秀惠指出前兩次留下的問題尚未形成後續，她要求團隊先列出可做到的下一步。")
            }
            _ => None,
        },
        8 => match (memory.information_thread, memory.team_trust) {
            (StoryThreadStage::Open, trust) if trust >= 55 => {
                Some("周以晴沿用上次公開的資料脈絡追問，並提醒團隊補上尚未確認的來源。")
            }
            (StoryThreadStage::Open, _) => {
                Some("周以晴逐項核對團隊先前的說法，先問原始資料何時能公開。")
            }
            (StoryThreadStage::Deferred, _) => {
                Some("周以晴記得團隊暫未回應片段爭議，這次把前次未答的問題重新帶回現場。")
            }
            _ => None,
        },
        12 => match (memory.information_thread, memory.reporter_trust) {
            (StoryThreadStage::FollowedUp, trust) if trust >= 60 => {
                Some("周以晴沿著已公開的查證紀錄做最後核對；尚未確認的空白也被列入追問。")
            }
            (StoryThreadStage::FollowedUp, _) => {
                Some("周以晴確認過團隊的補充資料，但仍要求候選人在最後一次公開回應中交代疑點。")
            }
            (StoryThreadStage::Deferred, _) => {
                Some("周以晴提醒團隊，前次沒有收束的問題仍在；辯論會可能再次被問到。")
            }
            _ => None,
        },
        _ => None,
    }
}

/// A short callback that carries the candidate's earlier story decisions into
/// the next campaign act. The daily scene remains shared, but this line makes
/// the candidate mode's route visibly responsive to prior choices.
pub fn daily_route_echo(week: u8, day: u8, progress: StoryProgress) -> Option<&'static str> {
    let decision_index = match (week, day) {
        (3, 1) => 0,
        (5, 1) => 1,
        (7, 1) => 2,
        (9, 1) => 3,
        (11, 1) => 4,
        (12, 2) => 5,
        _ => return None,
    };
    let decision = progress.decision(decision_index)?;
    Some(match (decision_index, decision) {
        (0, StoryDecision::Chosen(StoryChoiceId::FieldVisit)) => {
            "陳秀惠把現勘紀錄放到候選人面前：「路口我陪你走過了，接下來請告訴居民何時回來。」"
        }
        (0, StoryDecision::Chosen(StoryChoiceId::PublishFacts)) => {
            "周以晴拿著公開資料追問更新時間；候選人把已確認與待查項目分開說，沒有把未知補成承諾。"
        }
        (0, StoryDecision::Chosen(StoryChoiceId::CoordinateNetwork)) => {
            "吳明哲帶來新的聯絡窗口，也帶來不同單位的回覆期限；候選人要求把下一次追蹤排進行程。"
        }
        (0, StoryDecision::Skipped) => {
            "這份陳情沒有形成共同處理方案。陳秀惠仍把副本留在總部，候選人今天必須決定是否親自回覆。"
        }
        (1, StoryDecision::Chosen(StoryChoiceId::PublishFullClip)) => {
            "完整影像與逐字稿已交給查證小組，林予安提醒候選人：公開材料後，後續每個說法都要能對得上原始紀錄。"
        }
        (1, StoryDecision::Chosen(StoryChoiceId::CandidateInterview)) => {
            "專訪播出後，候選人收到支持與質疑並存的回饋；林予安把尚未回答的問題排進下一次行程。"
        }
        (1, StoryDecision::Chosen(StoryChoiceId::IgnoreClip)) => {
            "團隊把行程留給地方組織，但周以晴仍在採訪群組追問片段脈絡；林予安把這項未答問題記進待辦。"
        }
        (1, StoryDecision::Skipped) => {
            "候選人沒有選擇回應方式，剪輯片段仍在流傳；總部開始收到需要逐一處理的查證問題。"
        }
        (2, StoryDecision::Chosen(StoryChoiceId::TrackProgress)) => {
            "公開追蹤表開始更新，周以晴逐項核對期限；候選人知道承諾從此有了能被對照的日期。"
        }
        (2, StoryDecision::Chosen(StoryChoiceId::PublicRoundtable)) => {
            "協調會結束後，居民與專業者仍有不同優先順序；候選人把未達共識的部分留在紀錄裡。"
        }
        (2, StoryDecision::Chosen(StoryChoiceId::JointOversight)) => {
            "跨黨追蹤名單送到各組，吳明哲提醒候選人：列管能讓問題被看見，也會讓每次延誤都留下紀錄。"
        }
        (2, StoryDecision::Skipped) => {
            "前次沒有決定後續追蹤方式，居民各自收到不同答案；候選人今天得先說清楚誰負責回覆。"
        }
        (3, StoryDecision::Chosen(StoryChoiceId::PublishTimeline)) => {
            "時間軸與查證進度上線，周以晴逐項核對來源；候選人要求團隊把仍待確認的地方也留在頁面上。"
        }
        (3, StoryDecision::Chosen(StoryChoiceId::OpenQuestions)) => {
            "公開問答結束後，林予安整理出仍未解答的問題；候選人決定下一次不只回答熟悉的題目。"
        }
        (3, StoryDecision::Chosen(StoryChoiceId::SpokespersonBriefing)) => {
            "發言人把統一說明送進媒體群組，候選人仍親自確認每一項可核對的事實，避免聲明超出資料範圍。"
        }
        (3, StoryDecision::Skipped) => {
            "查證工作沒有固定更新節奏，外界各自拼湊片段；競選團隊仍可在最後一週選擇如何交代。"
        }
        (4, StoryDecision::Chosen(StoryChoiceId::VerifiedDeadline)) => {
            "候選人把改善期限和仍待處理事項一併公開；陳秀惠收起清單前，先圈出下次檢視日期。"
        }
        (4, StoryDecision::Chosen(StoryChoiceId::ResidentReinspection)) => {
            "居民代表與團隊走回現場，照片裡有改善，也有沒解決的地方；候選人沒有要求刪掉任何一張。"
        }
        (4, StoryDecision::Chosen(StoryChoiceId::PublishOpenItems)) => {
            "候選人把尚未完成的項目列在聲明最前面；掌聲不多，但周以晴找不到被藏起來的空白。"
        }
        (4, StoryDecision::Skipped) => {
            "最後一段陳情線沒有新的承諾，未完成事項仍留在桌上；選舉結束後，居民還會回來問進度。"
        }
        (5, StoryDecision::Chosen(StoryChoiceId::DebateWithRecord)) => {
            "辯論會上，候選人把完整紀錄攤開逐點回應；結束後林予安把仍待查的問題交給資料團隊持續更新。"
        }
        (5, StoryDecision::Chosen(StoryChoiceId::InterviewAfterDebate)) => {
            "候選人在辯論後接受專訪，沒有把不同意見說成已經消失；團隊把逐題說明留給選民自行檢視。"
        }
        (5, StoryDecision::Chosen(StoryChoiceId::PivotToTurnout)) => {
            "最後幾天，團隊把人力轉回催票與交通安排；仍有選民保留疑問，候選人選擇讓紀錄而非新口號留下。"
        }
        (5, StoryDecision::Skipped) => {
            "最後一場回應沒有安排，選戰轉入投票日準備；媒體與選民仍可依公開材料自行判讀。"
        }
        _ => return None,
    })
}

pub fn ending_note(progress: StoryProgress) -> String {
    let rain_ending = match (
        progress.decision(0),
        progress.decision(2),
        progress.decision(4),
    ) {
        (
            Some(StoryDecision::Chosen(StoryChoiceId::FieldVisit)),
            Some(StoryDecision::Chosen(StoryChoiceId::PublicRoundtable)),
            Some(StoryDecision::Chosen(StoryChoiceId::ResidentReinspection)),
        ) => "雨後陳情線：從現勘走到共同復勘，居民的通報清單交回下一個負責窗口。",
        (
            Some(StoryDecision::Chosen(StoryChoiceId::PublishFacts)),
            Some(StoryDecision::Chosen(StoryChoiceId::TrackProgress)),
            Some(StoryDecision::Chosen(StoryChoiceId::VerifiedDeadline)),
        ) => "雨後陳情線：資訊公開、進度追蹤和期限接在一起，尚未完成的部分仍保留在答案裡。",
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::VerifiedDeadline))) => {
            "雨後陳情線：候選人以可查核期限收尾，未完成項目都有下一個回覆日期。"
        }
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::ResidentReinspection))) => {
            "雨後陳情線：居民共同參與最後一次復勘，團隊帶走改善紀錄與待處理問題。"
        }
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::PublishOpenItems))) => {
            "雨後陳情線：候選人公開未解事項，並留下後續追蹤窗口。"
        }
        (_, _, Some(StoryDecision::Skipped)) => {
            "雨後陳情線：最後一段沒有作出新承諾，追蹤工作仍等待選後接手。"
        }
        _ => "雨後陳情線：留下的通報與待確認事項仍需交代。",
    };
    let clip_ending = match (
        progress.decision(1),
        progress.decision(3),
        progress.decision(5),
    ) {
        (
            Some(StoryDecision::Chosen(StoryChoiceId::PublishFullClip)),
            Some(StoryDecision::Chosen(StoryChoiceId::PublishTimeline)),
            Some(StoryDecision::Chosen(StoryChoiceId::DebateWithRecord)),
        ) => "剪輯查證線：從完整影像走到公開時間軸，候選人最後帶著紀錄回到辯論臺。",
        (
            Some(StoryDecision::Chosen(StoryChoiceId::CandidateInterview)),
            Some(StoryDecision::Chosen(StoryChoiceId::OpenQuestions)),
            Some(StoryDecision::Chosen(StoryChoiceId::InterviewAfterDebate)),
        ) => "剪輯查證線：候選人以多次公開問答回應質疑，讓仍未解答的問題也留在紀錄中。",
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::PivotToTurnout))) => {
            "剪輯查證線：最後把資源轉回投票日準備，並讓公開材料保留不同解讀的空間。"
        }
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::DebateWithRecord))) => {
            "剪輯查證線：候選人帶著現有紀錄在辯論會回應，未確認部分仍待後續查證。"
        }
        (_, _, Some(StoryDecision::Chosen(StoryChoiceId::InterviewAfterDebate))) => {
            "剪輯查證線：辯論後以專訪補充說明，團隊保留仍待確認的問題。"
        }
        (_, _, Some(StoryDecision::Skipped)) => {
            "剪輯查證線：最後一段沒有新增回應，公開材料留給選民自行判讀。"
        }
        _ => "剪輯查證線：仍有尚待說明的脈絡與查證問題。",
    };
    format!("{rain_ending} {clip_ending}")
}

pub fn choice_for(beat_index: usize, choice_index: usize) -> Option<&'static StoryChoiceSpec> {
    choices_for(beat_index)?.get(choice_index)
}

fn choices_for(beat_index: usize) -> Option<&'static [StoryChoiceSpec]> {
    Some(match beat_index {
        0 => OPENING_CHOICES,
        1 => CLIP_OPENING_CHOICES,
        2 => FOLLOWUP_CHOICES,
        3 => CLIP_FOLLOWUP_CHOICES,
        4 => CLOSING_CHOICES,
        5 => CLIP_CLOSING_CHOICES,
        _ => return None,
    })
}

pub fn coherence_bonus(
    beat_index: usize,
    choice: StoryChoiceId,
    progress: StoryProgress,
) -> StoryBonus {
    let route_bonus = if beat_index == 4 {
        match (progress.decision(0), progress.decision(2), choice) {
            (
                Some(StoryDecision::Chosen(StoryChoiceId::FieldVisit)),
                Some(StoryDecision::Chosen(StoryChoiceId::PublicRoundtable)),
                StoryChoiceId::ResidentReinspection,
            ) => StoryBonus {
                support_delta: 0.12,
                trust_delta: 0.5,
                note: Some("前後一致的現勘與居民協作路線，獲得信任加成。"),
            },
            (
                Some(StoryDecision::Chosen(StoryChoiceId::PublishFacts)),
                Some(StoryDecision::Chosen(StoryChoiceId::TrackProgress)),
                StoryChoiceId::VerifiedDeadline,
            ) => StoryBonus {
                support_delta: 0.12,
                trust_delta: 0.5,
                note: Some("前後一致的資訊公開路線，獲得信任加成。"),
            },
            (
                Some(StoryDecision::Chosen(StoryChoiceId::CoordinateNetwork)),
                Some(StoryDecision::Chosen(StoryChoiceId::JointOversight)),
                StoryChoiceId::PublishOpenItems,
            ) => StoryBonus {
                support_delta: 0.12,
                trust_delta: 0.5,
                note: Some("前後一致的地方協調與監督路線，獲得信任加成。"),
            },
            _ => StoryBonus::default(),
        }
    } else if beat_index == 5 {
        match (progress.decision(1), progress.decision(3), choice) {
            (
                Some(StoryDecision::Chosen(StoryChoiceId::PublishFullClip)),
                Some(StoryDecision::Chosen(StoryChoiceId::PublishTimeline)),
                StoryChoiceId::DebateWithRecord,
            ) => StoryBonus {
                support_delta: 0.12,
                trust_delta: 0.6,
                note: Some("前後一致的公開查證路線，獲得信任加成。"),
            },
            (
                Some(StoryDecision::Chosen(StoryChoiceId::CandidateInterview)),
                Some(StoryDecision::Chosen(StoryChoiceId::OpenQuestions)),
                StoryChoiceId::InterviewAfterDebate,
            ) => StoryBonus {
                support_delta: 0.10,
                trust_delta: 0.5,
                note: Some("持續接受追問並保留紀錄，獲得小幅連貫加成。"),
            },
            (
                Some(StoryDecision::Chosen(StoryChoiceId::IgnoreClip)),
                Some(StoryDecision::Chosen(StoryChoiceId::SpokespersonBriefing)),
                StoryChoiceId::PivotToTurnout,
            ) => StoryBonus {
                support_delta: 0.06,
                trust_delta: 0.2,
                note: Some("集中最後衝刺並完成團隊交接，獲得小幅節奏加成。"),
            },
            _ => StoryBonus::default(),
        }
    } else {
        StoryBonus::default()
    };
    let memory_bonus = match beat_index {
        4 if progress.memory.resident_trust >= 60 => StoryBonus {
            support_delta: 0.04,
            trust_delta: 0.25,
            note: Some("居民代表記得團隊的持續回訪，角色信任帶來小幅收尾加成。"),
        },
        5 if progress.memory.reporter_trust >= 55 => StoryBonus {
            support_delta: 0.04,
            trust_delta: 0.25,
            note: Some("記者沿用既有查證脈絡追問，穩定回應帶來小幅收尾加成。"),
        },
        _ => StoryBonus::default(),
    };
    StoryBonus {
        support_delta: route_bonus.support_delta + memory_bonus.support_delta,
        trust_delta: route_bonus.trust_delta + memory_bonus.trust_delta,
        note: route_bonus.note.or(memory_bonus.note),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_story_arcs_have_six_separated_beats_and_branching_scenes() {
        let mut progress = StoryProgress::default();
        let opening = beat_for_round(1, progress).unwrap();
        assert_eq!(opening.week, 2);
        assert_eq!(opening.choices.len(), 3);
        assert!(progress.record(0, StoryDecision::Chosen(StoryChoiceId::PublishFacts)));

        let clip_opening = beat_for_round(3, progress).unwrap();
        assert_eq!(clip_opening.week, 4);
        assert_eq!(clip_opening.choices.len(), 3);
        assert!(clip_opening.scene.contains("剪成短片"));
        assert!(progress.record(1, StoryDecision::Chosen(StoryChoiceId::PublishFullClip)));

        let followup = beat_for_round(5, progress).unwrap();
        assert_eq!(followup.week, 6);
        assert!(followup.scene.contains("已公開的資料"));
        assert!(progress.record(2, StoryDecision::Chosen(StoryChoiceId::TrackProgress)));

        let clip_followup = beat_for_round(7, progress).unwrap();
        assert_eq!(clip_followup.week, 8);
        assert!(clip_followup.scene.contains("完整影像與逐字稿"));
        assert!(progress.record(3, StoryDecision::Chosen(StoryChoiceId::PublishTimeline)));

        let closing = beat_for_round(9, progress).unwrap();
        assert_eq!(closing.week, 10);
        assert!(closing.scene.contains("公開追蹤表"));
        assert!(progress.record(4, StoryDecision::Chosen(StoryChoiceId::VerifiedDeadline)));

        let clip_closing = beat_for_round(11, progress).unwrap();
        assert_eq!(clip_closing.week, 12);
        assert!(clip_closing.scene.contains("查證時間軸"));
        assert_eq!(clip_closing.choices.len(), 3);
        assert!(beat_for_round(10, progress).is_none());
    }

    #[test]
    fn coherent_story_routes_gain_small_closing_bonuses_only() {
        let mut progress = StoryProgress::default();
        progress.record(0, StoryDecision::Chosen(StoryChoiceId::FieldVisit));
        progress.record(2, StoryDecision::Chosen(StoryChoiceId::PublicRoundtable));

        let coherent = coherence_bonus(4, StoryChoiceId::ResidentReinspection, progress);
        assert_eq!(coherent.support_delta, 0.16);
        assert_eq!(coherent.trust_delta, 0.75);
        assert_eq!(
            coherence_bonus(4, StoryChoiceId::PublishOpenItems, progress),
            StoryBonus {
                support_delta: 0.04,
                trust_delta: 0.25,
                note: Some("居民代表記得團隊的持續回訪，角色信任帶來小幅收尾加成。"),
            }
        );
        assert_eq!(
            coherence_bonus(0, StoryChoiceId::FieldVisit, progress),
            StoryBonus::default()
        );

        progress.record(1, StoryDecision::Chosen(StoryChoiceId::PublishFullClip));
        progress.record(3, StoryDecision::Chosen(StoryChoiceId::PublishTimeline));
        let clip_bonus = coherence_bonus(5, StoryChoiceId::DebateWithRecord, progress);
        assert_eq!(clip_bonus.support_delta, 0.16);
        assert_eq!(clip_bonus.trust_delta, 0.85);
    }

    #[test]
    fn candidate_daily_callbacks_and_ending_replay_the_chosen_route() {
        let mut progress = StoryProgress::default();
        assert!(daily_route_echo(3, 1, progress).is_none());
        progress.record(0, StoryDecision::Chosen(StoryChoiceId::FieldVisit));
        assert!(daily_route_echo(3, 1, progress)
            .unwrap()
            .contains("現勘紀錄"));
        progress.record(1, StoryDecision::Chosen(StoryChoiceId::PublishFullClip));
        assert!(daily_route_echo(5, 1, progress)
            .unwrap()
            .contains("完整影像"));
        progress.record(2, StoryDecision::Chosen(StoryChoiceId::PublicRoundtable));
        assert!(daily_route_echo(7, 1, progress).unwrap().contains("協調會"));
        progress.record(3, StoryDecision::Chosen(StoryChoiceId::PublishTimeline));
        assert!(daily_route_echo(9, 1, progress).unwrap().contains("時間軸"));
        progress.record(
            4,
            StoryDecision::Chosen(StoryChoiceId::ResidentReinspection),
        );
        assert!(daily_route_echo(11, 1, progress)
            .unwrap()
            .contains("居民代表"));
        progress.record(5, StoryDecision::Chosen(StoryChoiceId::DebateWithRecord));
        assert!(daily_route_echo(12, 2, progress)
            .unwrap()
            .contains("辯論會"));
        let ending = ending_note(progress);
        assert!(ending.contains("共同復勘"));
        assert!(ending.contains("辯論臺"));
        assert!(daily_route_echo(12, 3, progress).is_none());
    }

    #[test]
    fn returning_characters_remember_trust_and_unresolved_story_threads() {
        let mut trusted = StoryProgress::default();
        trusted.record(0, StoryDecision::Chosen(StoryChoiceId::FieldVisit));
        assert_eq!(trusted.memory().rain_thread, StoryThreadStage::Open);
        assert!(character_memory_echo(6, trusted)
            .unwrap()
            .contains("願意先一起核對"));
        trusted.record(2, StoryDecision::Chosen(StoryChoiceId::PublicRoundtable));
        assert_eq!(trusted.memory().rain_thread, StoryThreadStage::FollowedUp);
        assert!(character_memory_echo(10, trusted)
            .unwrap()
            .contains("共同整理的追蹤表"));
        trusted.record(
            4,
            StoryDecision::Chosen(StoryChoiceId::ResidentReinspection),
        );
        assert_eq!(trusted.memory().rain_thread, StoryThreadStage::Resolved);
        assert!(trusted.memory_summary().contains("已收束"));

        let mut deferred = StoryProgress::default();
        deferred.record(0, StoryDecision::Skipped);
        assert_eq!(deferred.memory().rain_thread, StoryThreadStage::Deferred);
        assert!(character_memory_echo(6, deferred)
            .unwrap()
            .contains("沒有承諾處理方式"));
        assert!(deferred.memory().resident_trust < trusted.memory().resident_trust);

        let mut reporter_route = StoryProgress::default();
        reporter_route.record(1, StoryDecision::Chosen(StoryChoiceId::PublishFullClip));
        assert!(character_memory_echo(8, reporter_route)
            .unwrap()
            .contains("沿用上次公開"));
        reporter_route.record(3, StoryDecision::Chosen(StoryChoiceId::PublishTimeline));
        assert_eq!(
            reporter_route.memory().information_thread,
            StoryThreadStage::FollowedUp
        );
        assert!(character_memory_echo(12, reporter_route)
            .unwrap()
            .contains("最後一次公開回應"));
    }
}
