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
}

impl LocalStoryProgress {
    pub const fn resident_followup_pending(self) -> bool {
        self.resident_followup_pending
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
            "{} {}（{}）｜{} {}（{}）｜{} {}（{}）｜居民承諾待追蹤：{}｜資訊誠信破口：{}",
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
            if self.integrity_breach { "是" } else { "否" }
        )
    }

    pub fn ending_reflection(self, region: &'static RegionProfile) -> String {
        if self.integrity_breach {
            format!(
                "{}的地方選戰仍留下資訊誠信待修復事項；選舉結果不會抹去居民與記者的追問。",
                region.name
            )
        } else if self.resident_followup_pending {
            format!(
                "{}的「{}」回訪承諾尚未結案，後續仍需要交代處理進度。",
                region.name, region.focus_label
            )
        } else if self.resident_trust >= 2 && self.reporter_trust >= 2 {
            format!(
                "{}的居民與地方記者都保留了追蹤管道；對「{}」的分歧仍待公共討論。",
                region.name, region.focus_label
            )
        } else {
            format!(
                "{}的「{}」議題已完成本輪交接，仍有部分居民期待更明確的後續說明。",
                region.name, region.focus_label
            )
        }
    }

    pub fn is_valid(self) -> bool {
        [
            self.resident_trust,
            self.organizer_trust,
            self.reporter_trust,
        ]
        .iter()
        .all(|trust| (-3..=3).contains(trust))
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
}
