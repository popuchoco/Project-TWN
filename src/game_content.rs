//! Fictional election, party, region, and event content for playable builds.
//!
//! Numeric values are scenario parameters. They do not report or reproduce
//! real election results, political organizations, people, or places.

pub use crate::local_content::{
    find_party, find_region, party_affinity, PartyProfile, RegionProfile, PARTY_PROFILES, REGIONS,
};
pub use crate::presidential_content::{ElectionScenario, ScenarioCandidate, ELECTIONS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Issue {
    Economy,
    Trust,
    CrossStrait,
    Media,
    Civic,
    Education,
    Rights,
    PublicHealth,
    Governance,
    Local,
}

#[derive(Clone, Copy, Debug)]
pub struct EventCard {
    pub year: u16,
    pub title: &'static str,
    pub description: &'static str,
    pub issue: Issue,
    pub active_campaign_years: &'static [u16],
}

pub const EVENTS: &[EventCard] = &[
    EventCard {
        year: 1994,
        title: "霧川北岸整建採購資料缺頁",
        description:
            "一項港區整建案的公開附件缺少版本紀錄，地方團隊必須決定先補齊資料、召開說明或暫停背書。",
        issue: Issue::Governance,
        active_campaign_years: &[1996, 2000, 2004],
    },
    EventCard {
        year: 1995,
        title: "南澗雨季接駁路線改動",
        description:
            "雨季施工使兩個社區的接駁時間改變；通勤者、店家與工程單位對優先順序有不同期待。",
        issue: Issue::Local,
        active_campaign_years: &[1996, 2000, 2004],
    },
    EventCard {
        year: 1997,
        title: "校園社團活動經費公開協調",
        description: "多個校園社團要求說明活動經費規則，校方與家長則關注安全及資源分配。",
        issue: Issue::Education,
        active_campaign_years: &[2000, 2004, 2008],
    },
    EventCard {
        year: 1998,
        title: "議會問政直播片段缺漏",
        description: "一段地方問政直播沒有完整存檔，各方對補件、重播與公開道歉的時機意見不一。",
        issue: Issue::Media,
        active_campaign_years: &[2000, 2004, 2008],
    },
    EventCard {
        year: 1999,
        title: "社區供水輪值規則重整",
        description: "枯水期用水輪值制度將調整，農戶、學校與觀光業者要求納入不同用水需求。",
        issue: Issue::PublicHealth,
        active_campaign_years: &[2000, 2004, 2008],
    },
    EventCard {
        year: 2003,
        title: "港區倉儲承攬資料疑義",
        description:
            "公共倉儲標案的評選紀錄出現版本落差；競選團隊可先查證，也可能因搶快而誤傳未確認訊息。",
        issue: Issue::Trust,
        active_campaign_years: &[2004, 2008, 2012],
    },
    EventCard {
        year: 2004,
        title: "夜間交通錄音剪輯爭議",
        description: "一段民眾提供的交通會議錄音在社群流傳，原始檔與剪輯段落尚待核對。",
        issue: Issue::Media,
        active_campaign_years: &[2004, 2008, 2012],
    },
    EventCard {
        year: 2007,
        title: "公共托育試辦名額排序",
        description: "新設托育據點名額有限，家庭、社區照護團體與地方承辦單位要求說明分配原則。",
        issue: Issue::Rights,
        active_campaign_years: &[2008, 2012, 2016],
    },
    EventCard {
        year: 2008,
        title: "連續降雨與抽水站調度",
        description: "連日降雨造成多處積水，地方必須在即時搶修、公開風險和後續預算之間排序。",
        issue: Issue::Local,
        active_campaign_years: &[2008, 2012, 2016],
    },
    EventCard {
        year: 2011,
        title: "外島交通減班與物資運輸",
        description: "天候與設備檢修使航班減少，居民、物流業者與行政單位協調替代班次。",
        issue: Issue::Economy,
        active_campaign_years: &[2012, 2016, 2020],
    },
    EventCard {
        year: 2012,
        title: "公共媒體專訪時段分配",
        description: "多組候選團隊爭取同一組專訪時段，製作單位公開分配規則後仍須回應公平性質疑。",
        issue: Issue::Media,
        active_campaign_years: &[2012, 2016, 2020],
    },
    EventCard {
        year: 2015,
        title: "公共閱讀資源採購原則協調",
        description:
            "跨校閱讀資源採購小組延長意見徵詢，家長、教師與圖書館員對適用範圍和評選程序提出不同建議。",
        issue: Issue::Education,
        active_campaign_years: &[2016, 2020, 2024],
    },
    EventCard {
        year: 2016,
        title: "校園無障礙修繕預算排序",
        description: "多所學校提出通行改善需求，地方必須說明先後順序及不同社區的公平性。",
        issue: Issue::Rights,
        active_campaign_years: &[2016, 2020, 2024],
    },
    EventCard {
        year: 2018,
        title: "沿海工業區空氣監測資料落差",
        description: "兩個監測站的讀值時間未同步，居民要求公開原始資料並說明後續健康追蹤方式。",
        issue: Issue::PublicHealth,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2019,
        title: "轉乘站影像脈絡查核",
        description: "一段公共設施影像經多次轉傳後失去拍攝時間與前後段落，涉及個資、查證及公共安全界線。",
        issue: Issue::Media,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2020,
        title: "校園接駁與訊息更新協調",
        description:
            "校舍整修後通學路線及通知管道需要短期調整，家長、校方與交通單位協調更新時程。",
        issue: Issue::Local,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2022,
        title: "運動中心外包契約重新檢視",
        description: "外包服務的驗收標準未能回答使用者的全部疑問，議會要求提出改約或重新招標方案。",
        issue: Issue::Governance,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2024,
        title: "地方檔案公開與個資界線",
        description:
            "居民要求公開補助評選紀錄，承辦團隊需在透明度、當事人隱私與可核對性之間取得平衡。",
        issue: Issue::Trust,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2025,
        title: "跨行政區防災物資協調",
        description: "多區同時提出物資需求，志工與公務團隊需要公開庫存、運送時程與排序原則。",
        issue: Issue::Civic,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2026,
        title: "沿海航運資訊中斷演練",
        description: "通訊演練期間，地方必須在安全提醒、物流替代與避免散布未證實消息之間做選擇。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2026],
    },
];

#[derive(Clone, Copy, Debug)]
pub struct AssemblySnapshot {
    pub year: u16,
    pub summary: &'static str,
}

pub const LEGISLATURE_SNAPSHOTS: &[AssemblySnapshot] = &[
    AssemblySnapshot { year: 1998, summary: "群島共治黨 54 席、海岳協作黨 42 席、其他聯盟 14 席；組織協商影響公共預算。" },
    AssemblySnapshot { year: 2004, summary: "曙光民主聯盟 48 席、群島共治黨 44 席、地方共好聯盟 18 席；未有單一聯盟過半。" },
    AssemblySnapshot { year: 2012, summary: "群島共治黨 51 席、海岳協作黨 37 席、公民新聲黨 16 席、其他 6 席。" },
    AssemblySnapshot { year: 2024, summary: "群島共治黨 39 席、曙光民主聯盟 38 席、公民新聲黨 15 席、地方共好聯盟 18 席；協商是組閣關鍵。" },
];

#[derive(Clone, Copy, Debug)]
pub struct VoterSegment {
    pub name: &'static str,
    pub weight_pct: f64,
    pub priority: Issue,
    pub responsiveness: f64,
    pub description: &'static str,
}

pub const VOTER_SEGMENTS: &[VoterSegment] = &[
    VoterSegment {
        name: "社區穩定優先者",
        weight_pct: 28.0,
        priority: Issue::Trust,
        responsiveness: 0.55,
        description: "較重視連續服務、可信度與投票提醒。",
    },
    VoterSegment {
        name: "政策比較者",
        weight_pct: 22.0,
        priority: Issue::Governance,
        responsiveness: 1.35,
        description: "對治理評價、候選人表現與議題回應較敏感。",
    },
    VoterSegment {
        name: "公共參與關注者",
        weight_pct: 14.0,
        priority: Issue::Civic,
        responsiveness: 1.20,
        description: "重視資訊公開、教育與公民參與機會。",
    },
    VoterSegment {
        name: "生活服務優先者",
        weight_pct: 16.0,
        priority: Issue::Local,
        responsiveness: 1.10,
        description: "關注交通、物價、照護和地方服務。",
    },
    VoterSegment {
        name: "區域安全關注者",
        weight_pct: 10.0,
        priority: Issue::CrossStrait,
        responsiveness: 1.15,
        description: "重視區域合作、安全風險與應變韌性。",
    },
    VoterSegment {
        name: "公平與照護倡議者",
        weight_pct: 10.0,
        priority: Issue::Rights,
        responsiveness: 1.05,
        description: "關注平等、教育、健康與社會安全網。",
    },
];

pub fn election_for_year(year: u16) -> Option<&'static ElectionScenario> {
    ELECTIONS.iter().find(|election| election.year == year)
}

pub fn baseline_election(year: u16) -> &'static ElectionScenario {
    ELECTIONS
        .iter()
        .rev()
        .find(|election| election.year <= year)
        .unwrap_or(&ELECTIONS[0])
}

pub fn events_for_campaign(year: u16) -> Vec<&'static EventCard> {
    EVENTS
        .iter()
        .filter(|event| event.active_campaign_years.contains(&year))
        .collect()
}

pub fn incumbent_candidate_index(election: &ElectionScenario) -> usize {
    election
        .candidates
        .iter()
        .position(|candidate| candidate.party == election.incumbent_party)
        .unwrap_or_else(|| {
            election
                .candidates
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.votes.cmp(&b.votes))
                .map(|(index, _)| index)
                .unwrap_or(0)
        })
}

pub fn challenger_candidate_index(election: &ElectionScenario) -> usize {
    election
        .candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| candidate.party != election.incumbent_party)
        .max_by(|(_, a), (_, b)| a.votes.cmp(&b.votes))
        .map(|(index, _)| index)
        .unwrap_or_else(|| incumbent_candidate_index(election))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fictional_election_scenarios_have_valid_synthetic_vote_vectors() {
        let mut candidate_ids = std::collections::HashSet::new();
        let mut years = std::collections::HashSet::new();
        for election in ELECTIONS {
            assert!(years.insert(election.year), "duplicate scenario year");
            let valid: u64 = election
                .candidates
                .iter()
                .map(|candidate| candidate.votes)
                .sum();
            let shares: f64 = election
                .candidates
                .iter()
                .map(|candidate| candidate.share_pct)
                .sum();
            assert!(valid > 0);
            assert!((shares - 100.0).abs() < 0.03);
            assert!(election.candidates.iter().all(|candidate| {
                !candidate.president.trim().is_empty()
                    && !candidate.vice_president.trim().is_empty()
                    && !candidate.party.trim().is_empty()
                    && candidate_ids.insert((candidate.president, candidate.vice_president))
            }));
        }
    }

    #[test]
    fn campaign_year_uses_latest_fictional_scenario_as_baseline() {
        assert_eq!(baseline_election(2026).year, 2024);
        assert_eq!(baseline_election(1995).year, 1996);
    }

    #[test]
    fn every_playable_cycle_has_synthetic_events_and_all_names_are_unique() {
        for year in [1996, 2000, 2004, 2008, 2012, 2016, 2020, 2024, 2026] {
            assert!(
                !events_for_campaign(year).is_empty(),
                "missing scenario events for {year}"
            );
        }
        let names: Vec<_> = REGIONS.iter().map(|region| region.name).collect();
        assert_eq!(names.len(), 22);
        assert_eq!(
            names.iter().collect::<std::collections::HashSet<_>>().len(),
            names.len()
        );
        let titles: Vec<_> = EVENTS.iter().map(|event| event.title).collect();
        assert_eq!(
            titles
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            titles.len()
        );
        assert!(EVENTS.iter().all(|event| {
            !event.description.trim().is_empty()
                && !event.active_campaign_years.is_empty()
                && event.active_campaign_years.iter().all(|year| {
                    events_for_campaign(*year)
                        .iter()
                        .any(|active| active.title == event.title)
                })
        }));
    }

    #[test]
    fn all_political_blocs_are_synthetic_and_available_for_local_play() {
        for party in PARTY_PROFILES {
            assert!(!party.name.trim().is_empty());
        }
        assert_eq!(
            PARTY_PROFILES
                .iter()
                .map(|party| party.name)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            PARTY_PROFILES.len()
        );
        assert!(find_region("霧川市").is_some());
        assert!(find_party("曙光民主聯盟").is_some());
        assert_eq!(
            VOTER_SEGMENTS
                .iter()
                .map(|segment| segment.weight_pct)
                .sum::<f64>(),
            100.0
        );
    }
}
