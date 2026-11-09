//! Fictional character memory and regional callbacks for local campaigns.

use crate::game_content::Issue;
use crate::local_content::RegionProfile;
use crate::sim::ActionId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LocalStoryProgress {
    resident_trust: i8,
    organizer_trust: i8,
    reporter_trust: i8,
    resident_followup_pending: bool,
    integrity_breach: bool,
    integrity_repaired: bool,
    decisions: [Option<LocalStoryDecision>; LOCAL_STORY_BEATS],
}

pub const LOCAL_STORY_BEATS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalStoryDecision {
    Chosen(u8),
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalStoryEnding {
    AccountablePartnership,
    CommunityCoalition,
    RepairedTrust,
    IntegrityAtRisk,
    UnfinishedCommitments,
    ContestedRelationship,
}

impl LocalStoryEnding {
    pub const fn label(self) -> &'static str {
        match self {
            Self::AccountablePartnership => "公開交代，建立追蹤",
            Self::CommunityCoalition => "跨社區協作成形",
            Self::RepairedTrust => "補正破口，修復信任",
            Self::IntegrityAtRisk => "資訊破口仍待修復",
            Self::UnfinishedCommitments => "承諾待續，留下交接",
            Self::ContestedRelationship => "關係未定，地方續談",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalStoryChoiceSpec {
    pub label: &'static str,
    pub description: &'static str,
    pub budget_cost: i32,
    pub support_delta: f64,
    pub trust_delta: f64,
    pub mobilization_delta: f64,
    pub stamina_delta: f64,
    resident_delta: i8,
    organizer_delta: i8,
    reporter_delta: i8,
    pub(crate) resolves_followup: bool,
    pub(crate) repairs_integrity: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalStoryBeat {
    pub index: usize,
    pub week: u8,
    pub title: &'static str,
    pub character: String,
    pub scene: String,
    pub choices: [LocalStoryChoiceSpec; 3],
}

impl LocalStoryProgress {
    pub const fn resident_followup_pending(self) -> bool {
        self.resident_followup_pending
    }

    pub fn decision(self, beat_index: usize) -> Option<LocalStoryDecision> {
        self.decisions.get(beat_index).copied().flatten()
    }

    pub fn completed_beats(self) -> u8 {
        self.decisions
            .iter()
            .filter(|decision| decision.is_some())
            .count() as u8
    }

    pub const fn has_unresolved_integrity_breach(self) -> bool {
        self.integrity_breach && !self.integrity_repaired
    }

    pub fn ending(self) -> LocalStoryEnding {
        if self.integrity_breach && !self.integrity_repaired {
            LocalStoryEnding::IntegrityAtRisk
        } else if self.resident_followup_pending {
            LocalStoryEnding::UnfinishedCommitments
        } else if self.integrity_breach && self.integrity_repaired {
            LocalStoryEnding::RepairedTrust
        } else if self.resident_trust >= 2 && self.reporter_trust >= 2 && !self.integrity_breach {
            LocalStoryEnding::AccountablePartnership
        } else if self.organizer_trust >= 2 && self.resident_trust >= 1 {
            LocalStoryEnding::CommunityCoalition
        } else {
            LocalStoryEnding::ContestedRelationship
        }
    }

    pub fn record_decision(
        &mut self,
        beat_index: usize,
        choice_index: Option<usize>,
        choice: Option<&LocalStoryChoiceSpec>,
    ) -> bool {
        if beat_index >= LOCAL_STORY_BEATS
            || self.decisions[beat_index].is_some()
            || self.decisions[..beat_index].iter().any(Option::is_none)
        {
            return false;
        }
        let decision = match choice_index {
            Some(index @ 0..=2) => LocalStoryDecision::Chosen(index as u8),
            None => LocalStoryDecision::Skipped,
            _ => return false,
        };
        if let (Some(spec), LocalStoryDecision::Chosen(_)) = (choice, decision) {
            adjust(&mut self.resident_trust, spec.resident_delta);
            adjust(&mut self.organizer_trust, spec.organizer_delta);
            adjust(&mut self.reporter_trust, spec.reporter_delta);
            if spec.resolves_followup {
                self.resident_followup_pending = false;
            }
            if spec.repairs_integrity && self.integrity_breach {
                self.integrity_repaired = true;
            }
        } else if matches!(decision, LocalStoryDecision::Skipped) {
            match beat_index {
                0 => adjust(&mut self.resident_trust, -1),
                1 => adjust(&mut self.reporter_trust, -1),
                2 => adjust(&mut self.organizer_trust, -1),
                _ => {}
            }
        } else {
            return false;
        }
        self.decisions[beat_index] = Some(decision);
        true
    }

    pub fn beat(&self, round: u8, region: &'static RegionProfile) -> Option<LocalStoryBeat> {
        let (index, week) = match round {
            2 => (0, 3),
            5 => (1, 6),
            8 => (2, 9),
            11 => (3, 12),
            _ => return None,
        };
        if self.decisions[index].is_some() || self.decisions[..index].iter().any(Option::is_none) {
            return None;
        }
        let cast = cast_for(region.focus);
        let (title, character, scene) = match index {
            0 => (
                "承諾要有回音",
                cast.resident,
                if self.resident_followup_pending {
                    format!("{}帶來「{}」陳情追蹤清單：上次留下的回覆期限到了，居民想知道哪些已完成、哪些還在處理。", cast.resident, region.focus_label)
                } else {
                    format!("{}再次談起「{}」：不同社區的需求不完全一樣，團隊得先確認現場，再決定如何回應。", cast.resident, region.focus_label)
                },
            ),
            1 => (
                "資料公開之後",
                cast.reporter,
                if self.integrity_breach && !self.integrity_repaired {
                    format!("{}帶著先前揭露的資料差異回來追問。如何補齊來源、修正說法，將決定「{}」議題能否回到可查證的討論。", cast.reporter, region.focus_label)
                } else {
                    format!("{}收到「{}」政見的初步資料，仍有幾筆需要對照。團隊得在公開速度與查證完整度之間作出安排。", cast.reporter, region.focus_label)
                },
            ),
            2 => (
                "各社區如何同行",
                cast.organizer,
                if self.organizer_trust >= 2 {
                    format!("{}整理了各地志工的排班表；大家願意協作，也希望「{}」行程保留社區自己的優先順序。", cast.organizer, region.focus_label)
                } else {
                    format!("{}回報外圍社區的志工排班出現缺口。團隊得決定如何安排「{}」行程，避免資源只集中在熟悉的街區。", cast.organizer, region.focus_label)
                },
            ),
            _ => (
                "投票日前的交接",
                cast.resident,
                if self.resident_followup_pending {
                    format!("投票日前夕，{}仍拿著未結案的「{}」清單。團隊可以留下清楚交接，也可以把最後資源放在催票；選舉不會自動完成承諾。", cast.resident, region.focus_label)
                } else if self.integrity_breach && !self.integrity_repaired {
                    format!("{}與{}都在等最後一次回覆。「{}」政見之外，尚未補正的資訊差異也留在選區記憶裡。", cast.resident, cast.reporter, region.focus_label)
                } else {
                    format!("{}與{}整理最後一份「{}」追蹤表。投票日前，團隊要選擇如何交代成果、交接窗口與仍待討論的分歧。", cast.resident, cast.organizer, region.focus_label)
                },
            ),
        };
        Some(LocalStoryBeat {
            index,
            week,
            title,
            character: character.to_string(),
            scene,
            choices: local_choices(index),
        })
    }

    pub fn record_action(
        &mut self,
        action: ActionId,
        local_match: bool,
        negative_tactic_detected: bool,
    ) -> Option<&'static str> {
        if negative_tactic_detected {
            adjust(&mut self.reporter_trust, -2);
            adjust(&mut self.resident_trust, -1);
            self.integrity_breach = true;
            self.integrity_repaired = false;
            return Some("操作遭查核揭露，地方記者要求完整交代；居民信任也受到影響。");
        }

        let mut note = None;
        if local_match {
            adjust(&mut self.resident_trust, 1);
            note = Some("行動呼應本區關注焦點，地方居民較願意提供後續回饋。");
        }

        match action {
            ActionId::CommunityTour => {
                adjust(&mut self.resident_trust, 1);
                if self.resident_followup_pending {
                    adjust(&mut self.resident_trust, 1);
                    self.resident_followup_pending = false;
                    note = Some("完成前次居民回訪並確認後續窗口，地方聯絡人已收到處理進度。")
                } else {
                    self.resident_followup_pending = true;
                    note = Some("收到新的地方陳情，已留下回訪期限與負責窗口。")
                }
            }
            ActionId::PolicyDelivery
            | ActionId::CrisisResponse
            | ActionId::PublicServices
            | ActionId::VulnerableCommunityCare
            | ActionId::DisasterResponseVisit => {
                adjust(&mut self.resident_trust, 1);
                if self.resident_followup_pending {
                    adjust(&mut self.resident_trust, 1);
                    self.resident_followup_pending = false;
                    note = Some("回報服務處理進度，履行居民前次提出的追蹤承諾。")
                } else {
                    note = Some("地方服務窗口完成進度回報，居民可向窗口查詢目前處理狀態。")
                }
            }
            ActionId::CampaignHeadquarters
            | ActionId::SupportersAssociation
            | ActionId::VolunteerCanvassing
            | ActionId::GroundTeamLeaflets
            | ActionId::StreetSweep
            | ActionId::TurnoutMobilization
            | ActionId::ReligiousCommunityVisit
            | ActionId::PilgrimageParticipation
            | ActionId::MassAttendance => {
                adjust(&mut self.organizer_trust, 1);
                note = Some("地方協力者完成分工回報；後續行程仍保留不同社區的自主安排。")
            }
            ActionId::TransparencyReport
            | ActionId::PolicyForum
            | ActionId::Debate
            | ActionId::MediaInterview
            | ActionId::PolicyPresentation
            | ActionId::FactCheckResponse
            | ActionId::OversightPetition
            | ActionId::PublicMessage => {
                adjust(&mut self.reporter_trust, 1);
                note = Some("地方採訪窗口收到可核對資料；尚未確認的部分仍標示待查。")
            }
            _ => {}
        }
        note
    }

    pub fn callback(self, week: u8, day: u8, region: &'static RegionProfile) -> Option<String> {
        if day != 1 {
            return None;
        }
        let cast = cast_for(region.focus);
        let scene = match week {
            3 if self.resident_followup_pending => format!(
                "{}帶來「{}」陳情的追蹤清單：團隊尚未把承諾回覆到每一個聯絡窗口。",
                cast.resident, region.focus_label
            ),
            3 if self.resident_trust >= 2 => format!(
                "{}核對完第一批回覆，也把新的「{}」需求交給同一個服務窗口。",
                cast.resident, region.focus_label
            ),
            3 => format!(
                "{}再次詢問「{}」的處理進度；她願意聽說明，但希望先看到具體時程。",
                cast.resident, region.focus_label
            ),
            6 if self.integrity_breach => format!(
                "{}拿出資料差異清單，追問團隊如何更正；「{}」的地方議題不能被未證實訊息蓋過。",
                cast.reporter, region.focus_label
            ),
            6 if self.reporter_trust >= 2 => format!(
                "{}確認收到「{}」的原始資料與更新時程，並把尚未確認的項目清楚標註。",
                cast.reporter, region.focus_label
            ),
            6 => format!(
                "{}要求補齊「{}」的資料來源；團隊得在採訪期限前交出可查核的版本。",
                cast.reporter, region.focus_label
            ),
            9 if self.organizer_trust >= 2 => format!(
                "{}把不同社區的行程表重新排在一起，志工願意繼續協作，也要求保留各地自己的優先順序。",
                cast.organizer
            ),
            9 => format!(
                "{}回報志工排班出現缺口；團隊必須說明「{}」議題行程如何照顧外圍社區。",
                cast.organizer, region.focus_label
            ),
            12 if self.resident_followup_pending => format!(
                "投票日前夕，{}仍拿著未回覆的「{}」清單；選舉不會自動完成這項承諾。",
                cast.resident, region.focus_label
            ),
            12 if self.integrity_breach => format!(
                "{}整理完最後一份採訪紀錄；「{}」的政見之外，查核揭露的資訊破口也留在選區記憶裡。",
                cast.reporter, region.focus_label
            ),
            12 if self.resident_trust >= 2 && self.reporter_trust >= 2 => format!(
                "{}與{}都保留了追蹤窗口；居民對「{}」仍有不同看法，但知道可以向誰詢問進度。",
                cast.resident, cast.reporter, region.focus_label
            ),
            12 => format!(
                "{}把未完成的「{}」事項交接給新一週的服務窗口；投票日前夕，地方工作仍要留下紀錄。",
                cast.resident, region.focus_label
            ),
            _ => return None,
        };
        Some(format!(
            "【{}｜{}】{}",
            region.name, region.focus_label, scene
        ))
    }

    pub fn summary(self, region: &'static RegionProfile) -> String {
        let cast = cast_for(region.focus);
        format!(
            "{} {}（{}）｜{} {}（{}）｜{} {}（{}）｜居民承諾待追蹤：{}｜資訊誠信破口：{}｜地方故事進度：{}/{}",
            cast.resident,
            self.resident_trust,
            trust_tone(self.resident_trust),
            cast.organizer,
            self.organizer_trust,
            trust_tone(self.organizer_trust),
            cast.reporter,
            self.reporter_trust,
            trust_tone(self.reporter_trust),
            if self.resident_followup_pending {
                "是"
            } else {
                "否"
            },
            if !self.integrity_breach {
                "否"
            } else if self.integrity_repaired {
                "曾發生、已補正"
            } else {
                "待修復"
            },
            self.completed_beats(),
            LOCAL_STORY_BEATS
        )
    }

    pub fn ending_reflection(self, region: &'static RegionProfile) -> String {
        let ending = self.ending();
        let reflection = match ending {
            LocalStoryEnding::IntegrityAtRisk => format!(
                "{}的地方選戰仍留下資訊誠信待修復事項；選舉結果不會抹去居民與記者的追問。",
                region.name
            ),
            LocalStoryEnding::UnfinishedCommitments => format!(
                "{}的「{}」回訪承諾尚未結案，後續仍需要交代處理進度。",
                region.name, region.focus_label
            ),
            LocalStoryEnding::RepairedTrust => format!(
                "{}已補上「{}」的資料來源與修正紀錄；資訊破口仍留在地方記憶中，後續也要維持可查核。",
                region.name, region.focus_label
            ),
            LocalStoryEnding::AccountablePartnership => format!(
                "{}的居民與地方記者都保留了追蹤管道；對「{}」的分歧仍待公共討論。",
                region.name, region.focus_label
            ),
            LocalStoryEnding::CommunityCoalition => format!(
                "{}的協力者已建立跨社區回報方式；「{}」仍由各地依自身需要持續協調。",
                region.name, region.focus_label
            ),
            LocalStoryEnding::ContestedRelationship => format!(
                "{}的「{}」議題已完成本輪交接，仍有部分居民期待更明確的後續說明。",
                region.name, region.focus_label
            ),
        };
        format!("{}：{}", ending.label(), reflection)
    }

    pub fn is_valid(self) -> bool {
        let trust_is_valid = [
            self.resident_trust,
            self.organizer_trust,
            self.reporter_trust,
        ]
        .iter()
        .all(|trust| (-3..=3).contains(trust));
        let mut found_gap = false;
        let mut valid_decisions = true;
        for decision in self.decisions {
            if decision.is_none() {
                found_gap = true;
            } else if found_gap {
                valid_decisions = false;
            }
            if matches!(decision, Some(LocalStoryDecision::Chosen(index)) if index > 2) {
                valid_decisions = false;
            }
        }
        trust_is_valid && valid_decisions && (!self.integrity_repaired || self.integrity_breach)
    }
}

fn local_choices(index: usize) -> [LocalStoryChoiceSpec; 3] {
    let choice = |label,
                  description,
                  budget_cost,
                  support_delta,
                  trust_delta,
                  mobilization_delta,
                  stamina_delta,
                  resident_delta,
                  organizer_delta,
                  reporter_delta,
                  resolves_followup,
                  repairs_integrity| LocalStoryChoiceSpec {
        label,
        description,
        budget_cost,
        support_delta,
        trust_delta,
        mobilization_delta,
        stamina_delta,
        resident_delta,
        organizer_delta,
        reporter_delta,
        resolves_followup,
        repairs_integrity,
    };
    match index {
        0 => [
            choice(
                "現勘後逐項回覆",
                "安排小組確認現場，逐項回報已完成與待辦。",
                4,
                0.10,
                0.9,
                0.2,
                -0.5,
                2,
                0,
                0,
                true,
                false,
            ),
            choice(
                "公布處理時程",
                "先公開窗口、期限與追蹤方式，讓居民能核對進度。",
                2,
                0.06,
                0.7,
                0.1,
                0.0,
                1,
                0,
                1,
                false,
                false,
            ),
            choice(
                "把資源轉向其他行程",
                "維持既定行程，把有限人力留給其他急迫工作。",
                0,
                0.03,
                -0.2,
                0.4,
                0.4,
                -1,
                0,
                0,
                false,
                false,
            ),
        ],
        1 => [
            choice(
                "公開來源並補正差異",
                "逐項補上來源、版本與修正紀錄；若有資訊破口，明確更正。",
                3,
                0.09,
                1.0,
                0.1,
                -0.3,
                0,
                0,
                2,
                false,
                true,
            ),
            choice(
                "辦一場公開說明會",
                "讓居民與記者直接提問，但需要調度場地與團隊人力。",
                5,
                0.08,
                0.8,
                0.4,
                -0.6,
                1,
                0,
                1,
                false,
                false,
            ),
            choice(
                "先完成獨立查核",
                "暫緩發布結論，先把資料交由團隊逐筆比對。",
                1,
                0.04,
                0.6,
                0.0,
                0.5,
                0,
                0,
                1,
                false,
                false,
            ),
        ],
        2 => [
            choice(
                "成立跨社區服務小組",
                "由各社區共同排定服務順序與回報窗口。",
                5,
                0.09,
                0.7,
                1.0,
                -0.4,
                1,
                2,
                0,
                false,
                false,
            ),
            choice(
                "保留各地自主排程",
                "共用資源與資訊，但由在地協力者決定拜訪順序。",
                2,
                0.06,
                0.8,
                0.5,
                0.0,
                1,
                1,
                0,
                false,
                false,
            ),
            choice(
                "集中人力衝刺掃街",
                "把團隊集中在主要街區，短期提升觸及與動員。",
                2,
                0.10,
                -0.3,
                1.3,
                -0.8,
                0,
                -1,
                0,
                false,
                false,
            ),
        ],
        _ => [
            choice(
                "公布成果與待辦交接表",
                "列出完成項目、責任窗口與後續期限，交由地方持續追蹤。",
                2,
                0.07,
                0.8,
                0.2,
                0.0,
                1,
                0,
                1,
                true,
                false,
            ),
            choice(
                "交由社區協力者接手",
                "把聯絡窗口與處理紀錄交接給地方團體，保留後續溝通空間。",
                1,
                0.04,
                0.9,
                0.5,
                0.2,
                1,
                1,
                0,
                true,
                false,
            ),
            choice(
                "最後一週全力催票",
                "加派人力投入催票行程，以短期動員換取較高體力與信任成本。",
                4,
                0.12,
                -0.3,
                1.5,
                -1.1,
                0,
                1,
                0,
                false,
                false,
            ),
        ],
    }
}

#[derive(Clone, Copy)]
struct LocalCast {
    resident: &'static str,
    organizer: &'static str,
    reporter: &'static str,
}

fn cast_for(issue: Issue) -> LocalCast {
    match issue {
        Issue::Economy => LocalCast {
            resident: "蘇芷庭",
            organizer: "廖啟南",
            reporter: "何沛珊",
        },
        Issue::Trust | Issue::Governance | Issue::Media => LocalCast {
            resident: "江孟秋",
            organizer: "邱承岳",
            reporter: "周以晴",
        },
        Issue::Education => LocalCast {
            resident: "林芷禾",
            organizer: "卓景澄",
            reporter: "許婉寧",
        },
        Issue::Rights => LocalCast {
            resident: "曾以晴",
            organizer: "羅祐晨",
            reporter: "顏婉君",
        },
        Issue::PublicHealth => LocalCast {
            resident: "葉安知",
            organizer: "楊昀亭",
            reporter: "沈思妤",
        },
        Issue::CrossStrait => LocalCast {
            resident: "洪以川",
            organizer: "蔡念庭",
            reporter: "宋語安",
        },
        Issue::Civic | Issue::Local => LocalCast {
            resident: "陳秀惠",
            organizer: "吳明哲",
            reporter: "周以晴",
        },
    }
}

fn adjust(value: &mut i8, delta: i8) {
    *value = (*value + delta).clamp(-3, 3);
}

fn trust_tone(value: i8) -> &'static str {
    match value {
        3..=i8::MAX => "信任穩固",
        1..=2 => "願意持續回應",
        0 => "關係尚待累積",
        -2..=-1 => "開始保持距離",
        _ => "信任明顯受損",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local_content::REGIONS;

    #[test]
    fn callbacks_are_localized_to_region_focus_and_only_fire_at_chapters() {
        let region = REGIONS
            .iter()
            .find(|region| region.name == "雲棲市")
            .unwrap();
        let mut progress = LocalStoryProgress::default();
        progress.record_action(ActionId::PolicyPresentation, true, false);

        assert!(progress.callback(3, 2, region).is_none());
        let callback = progress.callback(6, 1, region).unwrap();
        assert!(callback.contains("雲棲市"));
        assert!(callback.contains("校園與青年公共服務"));
        assert!(callback.contains("許婉寧"));
    }

    #[test]
    fn local_service_actions_resolve_community_promises_and_exposed_tactics_break_trust() {
        let mut progress = LocalStoryProgress::default();
        progress.record_action(ActionId::CommunityTour, false, false);
        assert!(progress.resident_followup_pending);
        let note = progress
            .record_action(ActionId::PublicServices, true, false)
            .unwrap();
        assert!(note.contains("履行居民前次提出"));
        assert!(!progress.resident_followup_pending);

        progress.record_action(ActionId::AnonymousSmear, false, true);
        assert!(progress.integrity_breach);
        assert!(progress.reporter_trust < 0);
        assert!(progress
            .ending_reflection(
                REGIONS
                    .iter()
                    .find(|region| region.name == "霧川市")
                    .unwrap()
            )
            .contains("資訊誠信待修復"));
    }

    #[test]
    fn relationship_memory_is_bounded_and_replayable_from_action_choices() {
        let mut first = LocalStoryProgress::default();
        let choices = [
            (ActionId::CommunityTour, true, false),
            (ActionId::PublicServices, true, false),
            (ActionId::VolunteerCanvassing, false, false),
            (ActionId::TransparencyReport, false, false),
        ];
        for (action, local_match, detected) in choices {
            first.record_action(action, local_match, detected);
        }
        let mut replay = LocalStoryProgress::default();
        for (action, local_match, detected) in choices {
            replay.record_action(action, local_match, detected);
        }
        assert_eq!(first, replay);
        assert!(first.is_valid());
        assert!(!first.resident_followup_pending);

        for _ in 0..8 {
            first.record_action(ActionId::VolunteerCanvassing, false, false);
        }
        assert_eq!(first.organizer_trust, 3);
    }

    #[test]
    fn regional_chapters_offer_choices_and_resolve_into_distinct_endings() {
        let region = REGIONS
            .iter()
            .find(|region| region.name == "雲棲市")
            .unwrap();
        let mut progress = LocalStoryProgress::default();
        let opening = progress.beat(2, region).unwrap();
        assert_eq!(opening.week, 3);
        assert_eq!(opening.choices.len(), 3);
        assert!(opening.scene.contains("校園與青年公共服務"));
        assert!(progress.beat(5, region).is_none());

        let first_choice = opening.choices[0].clone();
        assert!(progress.record_decision(0, Some(0), Some(&first_choice)));
        let evidence = progress.beat(5, region).unwrap();
        assert_eq!(evidence.index, 1);
        let repair_choice = evidence.choices[0].clone();
        progress.record_action(ActionId::AnonymousSmear, false, true);
        assert!(progress.has_unresolved_integrity_breach());
        assert!(progress.record_decision(1, Some(0), Some(&repair_choice)));
        assert!(!progress.has_unresolved_integrity_breach());
        assert_eq!(progress.ending(), LocalStoryEnding::RepairedTrust);

        let community = progress.beat(8, region).unwrap();
        let team_choice = community.choices[0].clone();
        assert!(progress.record_decision(2, Some(0), Some(&team_choice)));
        let finale = progress.beat(11, region).unwrap();
        let handoff = finale.choices[1].clone();
        assert!(progress.record_decision(3, Some(1), Some(&handoff)));
        assert_eq!(progress.completed_beats(), 4);
        assert!(progress.is_valid());
        assert!(progress
            .ending_reflection(region)
            .contains("補正破口，修復信任"));
    }

    #[test]
    fn regional_story_decisions_must_follow_chapter_order_and_can_be_skipped() {
        let region = REGIONS
            .iter()
            .find(|region| region.name == "霧川市")
            .unwrap();
        let mut progress = LocalStoryProgress::default();
        let later_choice = local_choices(1)[0].clone();
        assert!(!progress.record_decision(1, Some(0), Some(&later_choice)));
        assert!(progress.record_decision(0, None, None));
        assert_eq!(progress.decision(0), Some(LocalStoryDecision::Skipped));
        assert!(progress.beat(5, region).is_some());
        assert!(progress.is_valid());
    }
}
