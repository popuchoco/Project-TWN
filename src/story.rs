//! Choice-driven campaign story content.
//!
//! The first story arc is a fictional composite scenario. It uses recurring
//! characters and consequences as game material; it does not depict a single
//! historical event or claim a fixed political effect.

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
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryDecision {
    Chosen(StoryChoiceId),
    Skipped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StoryProgress {
    decisions: [Option<StoryDecision>; 3],
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

    pub(crate) fn record(&mut self, beat_index: usize, decision: StoryDecision) -> bool {
        let Some(slot) = self.decisions.get_mut(beat_index) else {
            return false;
        };
        if slot.is_some() {
            return false;
        }
        *slot = Some(decision);
        true
    }

    pub(crate) fn is_valid(self, round: u8) -> bool {
        let trigger_rounds = [1_u8, 5, 9];
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

pub fn beat_for_round(round: u8, progress: StoryProgress) -> Option<StoryBeat> {
    let (index, week, character, scene, choices) = match round {
        1 => (
            0,
            2,
            "陳秀惠（虛構里民代表）",
            "一場豪雨後，幾個路口積水，公車也得臨時改道。里民代表陳秀惠帶來通報清單；競選幕僚林予安請團隊決定先從哪裡著手。",
            OPENING_CHOICES,
        ),
        5 => (
            1,
            6,
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
        9 => (
            2,
            10,
            "陳秀惠（虛構里民代表）",
            match progress.decision(1) {
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
        _ => return None,
    };

    if progress.decision(index).is_some() {
        return None;
    }
    Some(StoryBeat {
        index,
        week,
        title: "雨後一條路：陳情、查證與回覆",
        character,
        scene,
        choices,
    })
}

pub fn choice_for(beat_index: usize, choice_index: usize) -> Option<&'static StoryChoiceSpec> {
    choices_for(beat_index)?.get(choice_index)
}

fn choices_for(beat_index: usize) -> Option<&'static [StoryChoiceSpec]> {
    Some(match beat_index {
        0 => OPENING_CHOICES,
        1 => FOLLOWUP_CHOICES,
        2 => CLOSING_CHOICES,
        _ => return None,
    })
}

pub fn coherence_bonus(
    beat_index: usize,
    choice: StoryChoiceId,
    progress: StoryProgress,
) -> StoryBonus {
    if beat_index != 2 {
        return StoryBonus::default();
    }

    let route = (progress.decision(0), progress.decision(1), choice);
    match route {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn story_arc_has_three_separated_beats_and_branching_scenes() {
        let mut progress = StoryProgress::default();
        let opening = beat_for_round(1, progress).unwrap();
        assert_eq!(opening.week, 2);
        assert_eq!(opening.choices.len(), 3);
        assert!(progress.record(0, StoryDecision::Chosen(StoryChoiceId::PublishFacts)));

        let followup = beat_for_round(5, progress).unwrap();
        assert_eq!(followup.week, 6);
        assert!(followup.scene.contains("已公開的資料"));
        assert!(progress.record(1, StoryDecision::Chosen(StoryChoiceId::TrackProgress)));

        let closing = beat_for_round(9, progress).unwrap();
        assert_eq!(closing.week, 10);
        assert!(closing.scene.contains("公開追蹤表"));
        assert!(beat_for_round(10, progress).is_none());
    }

    #[test]
    fn coherent_story_routes_gain_a_small_closing_bonus_only() {
        let mut progress = StoryProgress::default();
        progress.record(0, StoryDecision::Chosen(StoryChoiceId::FieldVisit));
        progress.record(1, StoryDecision::Chosen(StoryChoiceId::PublicRoundtable));

        let coherent = coherence_bonus(2, StoryChoiceId::ResidentReinspection, progress);
        assert_eq!(coherent.support_delta, 0.12);
        assert_eq!(coherent.trust_delta, 0.5);
        assert_eq!(
            coherence_bonus(2, StoryChoiceId::PublishOpenItems, progress),
            StoryBonus::default()
        );
        assert_eq!(
            coherence_bonus(0, StoryChoiceId::FieldVisit, progress),
            StoryBonus::default()
        );
    }
}
