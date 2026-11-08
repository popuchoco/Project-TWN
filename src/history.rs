//! Historical election results and a neutral event timeline.
//!
//! Vote totals and percentages below are transcribed from Taiwan's Central
//! Election Commission (CEC). Events are context cards, not claims that an
//! event mechanically caused a particular election result.

#[derive(Clone, Copy, Debug)]
pub struct CandidateResult {
    pub president: &'static str,
    pub vice_president: &'static str,
    pub party: &'static str,
    pub votes: u64,
    pub share_pct: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct PresidentialElection {
    pub year: u16,
    pub date: &'static str,
    pub incumbent_party: &'static str,
    pub candidates: &'static [CandidateResult],
}

pub const ELECTIONS: &[PresidentialElection] = &[
    PresidentialElection {
        year: 1996,
        date: "1996-03-23",
        incumbent_party: "國民黨",
        candidates: &[
            CandidateResult {
                president: "李登輝",
                vice_president: "連戰",
                party: "國民黨",
                votes: 5_813_699,
                share_pct: 54.00,
            },
            CandidateResult {
                president: "彭明敏",
                vice_president: "謝長廷",
                party: "民進黨",
                votes: 2_274_586,
                share_pct: 21.13,
            },
            CandidateResult {
                president: "林洋港",
                vice_president: "郝柏村",
                party: "無黨籍／連線",
                votes: 1_603_790,
                share_pct: 14.90,
            },
            CandidateResult {
                president: "陳履安",
                vice_president: "王清峰",
                party: "無黨籍／連線",
                votes: 1_074_044,
                share_pct: 9.98,
            },
        ],
    },
    PresidentialElection {
        year: 2000,
        date: "2000-03-18",
        incumbent_party: "國民黨",
        candidates: &[
            CandidateResult {
                president: "陳水扁",
                vice_president: "呂秀蓮",
                party: "民進黨",
                votes: 4_977_697,
                share_pct: 39.30,
            },
            CandidateResult {
                president: "宋楚瑜",
                vice_president: "張昭雄",
                party: "無黨籍／連線",
                votes: 4_664_972,
                share_pct: 36.84,
            },
            CandidateResult {
                president: "連戰",
                vice_president: "蕭萬長",
                party: "國民黨",
                votes: 2_925_513,
                share_pct: 23.10,
            },
            CandidateResult {
                president: "許信良",
                vice_president: "朱慧良",
                party: "無黨籍／連線",
                votes: 79_429,
                share_pct: 0.63,
            },
            CandidateResult {
                president: "李敖",
                vice_president: "馮滬祥",
                party: "新黨",
                votes: 16_782,
                share_pct: 0.13,
            },
        ],
    },
    PresidentialElection {
        year: 2004,
        date: "2004-03-20",
        incumbent_party: "民進黨",
        candidates: &[
            CandidateResult {
                president: "陳水扁",
                vice_president: "呂秀蓮",
                party: "民進黨",
                votes: 6_471_970,
                share_pct: 50.11,
            },
            CandidateResult {
                president: "連戰",
                vice_president: "宋楚瑜",
                party: "國民黨／親民黨",
                votes: 6_442_452,
                share_pct: 49.89,
            },
        ],
    },
    PresidentialElection {
        year: 2008,
        date: "2008-03-22",
        incumbent_party: "民進黨",
        candidates: &[
            CandidateResult {
                president: "馬英九",
                vice_president: "蕭萬長",
                party: "國民黨",
                votes: 7_659_014,
                share_pct: 58.45,
            },
            CandidateResult {
                president: "謝長廷",
                vice_president: "蘇貞昌",
                party: "民進黨",
                votes: 5_444_949,
                share_pct: 41.55,
            },
        ],
    },
    PresidentialElection {
        year: 2012,
        date: "2012-01-14",
        incumbent_party: "國民黨",
        candidates: &[
            CandidateResult {
                president: "馬英九",
                vice_president: "吳敦義",
                party: "國民黨",
                votes: 6_891_139,
                share_pct: 51.60,
            },
            CandidateResult {
                president: "蔡英文",
                vice_president: "蘇嘉全",
                party: "民進黨",
                votes: 6_093_578,
                share_pct: 45.63,
            },
            CandidateResult {
                president: "宋楚瑜",
                vice_president: "林瑞雄",
                party: "親民黨",
                votes: 369_588,
                share_pct: 2.77,
            },
        ],
    },
    PresidentialElection {
        year: 2016,
        date: "2016-01-16",
        incumbent_party: "國民黨",
        candidates: &[
            CandidateResult {
                president: "蔡英文",
                vice_president: "陳建仁",
                party: "民進黨",
                votes: 6_894_744,
                share_pct: 56.12,
            },
            CandidateResult {
                president: "朱立倫",
                vice_president: "王如玄",
                party: "國民黨",
                votes: 3_813_365,
                share_pct: 31.04,
            },
            CandidateResult {
                president: "宋楚瑜",
                vice_president: "徐欣瑩",
                party: "親民黨",
                votes: 1_576_861,
                share_pct: 12.84,
            },
        ],
    },
    PresidentialElection {
        year: 2020,
        date: "2020-01-11",
        incumbent_party: "民進黨",
        candidates: &[
            CandidateResult {
                president: "蔡英文",
                vice_president: "賴清德",
                party: "民進黨",
                votes: 8_170_231,
                share_pct: 57.13,
            },
            CandidateResult {
                president: "韓國瑜",
                vice_president: "張善政",
                party: "國民黨",
                votes: 5_522_119,
                share_pct: 38.61,
            },
            CandidateResult {
                president: "宋楚瑜",
                vice_president: "余湘",
                party: "親民黨",
                votes: 608_590,
                share_pct: 4.26,
            },
        ],
    },
    PresidentialElection {
        year: 2024,
        date: "2024-01-13",
        incumbent_party: "民進黨",
        candidates: &[
            CandidateResult {
                president: "賴清德",
                vice_president: "蕭美琴",
                party: "民進黨",
                votes: 5_586_019,
                share_pct: 40.05,
            },
            CandidateResult {
                president: "侯友宜",
                vice_president: "趙少康",
                party: "國民黨",
                votes: 4_671_021,
                share_pct: 33.49,
            },
            CandidateResult {
                president: "柯文哲",
                vice_president: "吳欣盈",
                party: "民眾黨",
                votes: 3_690_466,
                share_pct: 26.46,
            },
        ],
    },
];

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

/// A selectable local-executive scenario. The 2022 winner party is a factual
/// reference; issue focus and starting poll balance are explicitly game rules.
#[derive(Clone, Copy, Debug)]
pub struct RegionProfile {
    pub key: &'static str,
    pub name: &'static str,
    pub previous_winner_party: &'static str,
    pub focus: Issue,
    pub focus_label: &'static str,
}

pub const REGIONS: &[RegionProfile] = &[
    RegionProfile {
        key: "taipei",
        name: "臺北市",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "居住、交通與城市治理",
    },
    RegionProfile {
        key: "new-taipei",
        name: "新北市",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "交通建設與公共服務",
    },
    RegionProfile {
        key: "taoyuan",
        name: "桃園市",
        previous_winner_party: "國民黨",
        focus: Issue::Economy,
        focus_label: "產業發展與交通",
    },
    RegionProfile {
        key: "taichung",
        name: "臺中市",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "交通、空品與城市建設",
    },
    RegionProfile {
        key: "tainan",
        name: "臺南市",
        previous_winner_party: "民進黨",
        focus: Issue::Economy,
        focus_label: "產業、農業與文化觀光",
    },
    RegionProfile {
        key: "kaohsiung",
        name: "高雄市",
        previous_winner_party: "民進黨",
        focus: Issue::Economy,
        focus_label: "港灣產業轉型與交通",
    },
    RegionProfile {
        key: "keelung",
        name: "基隆市",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "港區、交通與觀光",
    },
    RegionProfile {
        key: "hsinchu-city",
        name: "新竹市",
        previous_winner_party: "民眾黨",
        focus: Issue::Governance,
        focus_label: "城市治理與公共服務",
    },
    RegionProfile {
        key: "hsinchu-county",
        name: "新竹縣",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "城鄉建設與交通",
    },
    RegionProfile {
        key: "miaoli",
        name: "苗栗縣",
        previous_winner_party: "無黨籍",
        focus: Issue::Economy,
        focus_label: "農業、觀光與財政",
    },
    RegionProfile {
        key: "changhua",
        name: "彰化縣",
        previous_winner_party: "國民黨",
        focus: Issue::Economy,
        focus_label: "農業、產業與空品",
    },
    RegionProfile {
        key: "nantou",
        name: "南投縣",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "觀光、交通與防災",
    },
    RegionProfile {
        key: "yunlin",
        name: "雲林縣",
        previous_winner_party: "國民黨",
        focus: Issue::Economy,
        focus_label: "農業、長照與產業",
    },
    RegionProfile {
        key: "chiayi-city",
        name: "嘉義市",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "公共服務與城市更新",
    },
    RegionProfile {
        key: "chiayi-county",
        name: "嘉義縣",
        previous_winner_party: "民進黨",
        focus: Issue::Economy,
        focus_label: "農業、人口與交通",
    },
    RegionProfile {
        key: "pingtung",
        name: "屏東縣",
        previous_winner_party: "民進黨",
        focus: Issue::Economy,
        focus_label: "農業、觀光與交通",
    },
    RegionProfile {
        key: "yilan",
        name: "宜蘭縣",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "觀光、交通與環境",
    },
    RegionProfile {
        key: "hualien",
        name: "花蓮縣",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "觀光、交通與災後韌性",
    },
    RegionProfile {
        key: "taitung",
        name: "臺東縣",
        previous_winner_party: "國民黨",
        focus: Issue::Local,
        focus_label: "交通、觀光與部落服務",
    },
    RegionProfile {
        key: "penghu",
        name: "澎湖縣",
        previous_winner_party: "民進黨",
        focus: Issue::Local,
        focus_label: "離島交通、觀光與漁業",
    },
    RegionProfile {
        key: "kinmen",
        name: "金門縣",
        previous_winner_party: "無黨籍",
        focus: Issue::CrossStrait,
        focus_label: "離島建設與兩岸往來",
    },
    RegionProfile {
        key: "lienchiang",
        name: "連江縣",
        previous_winner_party: "國民黨",
        focus: Issue::CrossStrait,
        focus_label: "離島交通、觀光與對外往來",
    },
];

pub fn find_region(query: &str) -> Option<&'static RegionProfile> {
    let normalized = normalize_region_name(query);
    REGIONS.iter().find(|region| {
        region.key.eq_ignore_ascii_case(query.trim())
            || normalize_region_name(region.name) == normalized
    })
}

fn normalize_region_name(name: &str) -> String {
    name.trim()
        .replace('台', "臺")
        .trim_end_matches('市')
        .trim_end_matches('縣')
        .to_string()
}

#[derive(Clone, Copy, Debug)]
pub struct EventCard {
    pub year: u16,
    pub title: &'static str,
    pub description: &'static str,
    pub issue: Issue,
    /// Campaign years for which the card can be drawn. The event remains
    /// available as historical context outside this window.
    pub active_campaign_years: &'static [u16],
}

pub const EVENTS: &[EventCard] = &[
    EventCard { year: 1996, title: "首次總統直選", description: "1996年3月23日首次由選民直接選出正副總統，民主選舉成為全國性政治競爭的制度基礎。", issue: Issue::Civic, active_campaign_years: &[2000] },
    EventCard { year: 2000, title: "首次中央政黨輪替", description: "民進黨候選人當選總統，臺灣中央行政權首次由國民黨移轉至另一政黨。", issue: Issue::Governance, active_campaign_years: &[2004] },
    EventCard { year: 2004, title: "三一九槍擊案與選後爭議", description: "投票前夕槍擊事件及其後的驗票、訴訟與社會爭議；模擬聚焦信任、資訊公開與降溫。", issue: Issue::Trust, active_campaign_years: &[2004] },
    EventCard { year: 2008, title: "陳雲林來台及抗議爭議", description: "兩岸協商與警方維安、集會表達自由之間引發公共爭論。", issue: Issue::CrossStrait, active_campaign_years: &[2012] },
    EventCard { year: 2012, title: "反媒體壟斷運動", description: "媒體集中與新聞自主成為公民團體和公共政策討論議題。", issue: Issue::Media, active_campaign_years: &[2016] },
    EventCard { year: 2014, title: "太陽花學運", description: "學生與公民團體以服貿審查程序、國會監督及兩岸經貿為主要訴求。", issue: Issue::Civic, active_campaign_years: &[2014, 2016] },
    EventCard { year: 2015, title: "反課綱微調運動", description: "高中課綱調整引發程序、史觀與教育參與的公共爭論。", issue: Issue::Education, active_campaign_years: &[2015, 2016] },
    EventCard { year: 2016, title: "執政與國會多數的責任考驗", description: "民進黨首次同時取得總統職位與立法院多數，讓政策執行與監督成為後續選戰的重要檢視面向。", issue: Issue::Governance, active_campaign_years: &[2020] },
    EventCard { year: 2018, title: "地方選舉與「韓流」現象", description: "2018年地方選舉中，韓國瑜在高雄勝選並帶動全國政治討論。研究將其傳播、訴求與支持擴張列為觀察面向；本作不把它設定成任何政黨的固定加成。", issue: Issue::Economy, active_campaign_years: &[2020] },
    EventCard { year: 2019, title: "同性婚姻法制化", description: "司法院釋字第748號解釋施行法於2019年5月24日施行；社會平權與價值政策進入制度實踐。", issue: Issue::Rights, active_campaign_years: &[2020, 2024] },
    EventCard { year: 2020, title: "COVID-19疫情治理", description: "臺灣於2020年1月21日公布首例境外移入確診；該日期晚於1月11日總統投票日，因此不作為2020總統票的選前因素，供後續選戰檢視治理經驗。", issue: Issue::PublicHealth, active_campaign_years: &[2024] },
    EventCard { year: 2022, title: "地方選舉的執政評價", description: "地方選舉結果提醒參選者：地方服務、候選人特質及中央執政評價可能彼此不同，勿將地方票直接等同全國民意。", issue: Issue::Local, active_campaign_years: &[2024] },
    EventCard { year: 2024, title: "三黨不過半的國會", description: "立法院席次由國民黨52、民進黨51、民眾黨8席及無黨籍2席組成；協商、監督與政策溝通成為執政議題。", issue: Issue::Governance, active_campaign_years: &[2026] },
    EventCard { year: 2026, title: "「洋流」選戰敘事與地方參與", description: "2026年媒體與競選活動以「洋流」討論青年參與、地方互動及支持擴散；其政治意義仍有不同解讀。本作將其作為可回應的公民參與議題，不預設特定陣營受益。", issue: Issue::Civic, active_campaign_years: &[2026] },
    EventCard { year: 2026, title: "地方公職人員選舉（尚未投票）", description: "中選會訂定投票日為2026年11月28日；截至2026年11月8日結果尚未產生。本作只提供情境模擬，不預填勝負。", issue: Issue::Local, active_campaign_years: &[2026] },
];

#[derive(Clone, Copy, Debug)]
pub struct LegislatureSnapshot {
    pub year: u16,
    pub summary: &'static str,
}

pub const LEGISLATURE_SNAPSHOTS: &[LegislatureSnapshot] = &[
    LegislatureSnapshot {
        year: 2016,
        summary: "民進黨68席、國民黨35席；民進黨首次取得立法院多數。",
    },
    LegislatureSnapshot {
        year: 2024,
        summary: "國民黨52席、民進黨51席、民眾黨8席、無黨籍2席；沒有單一政黨過半。",
    },
];

pub fn election_for_year(year: u16) -> Option<&'static PresidentialElection> {
    ELECTIONS.iter().find(|election| election.year == year)
}

/// Return the most recent completed presidential election no later than `year`.
/// 2026's local-election scenario intentionally uses the 2024 presidential
/// result only as a clearly labelled national-level proxy.
pub fn baseline_election(year: u16) -> &'static PresidentialElection {
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

pub fn incumbent_candidate_index(election: &PresidentialElection) -> usize {
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

pub fn challenger_candidate_index(election: &PresidentialElection) -> usize {
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
    fn historical_votes_sum_to_valid_votes() {
        for election in ELECTIONS {
            let valid: u64 = election
                .candidates
                .iter()
                .map(|candidate| candidate.votes)
                .sum();
            assert!(valid > 0, "{} has no valid votes", election.year);
            let share_sum: f64 = election
                .candidates
                .iter()
                .map(|candidate| candidate.share_pct)
                .sum();
            assert!(
                (share_sum - 100.0).abs() < 0.03,
                "{} shares sum to {share_sum}",
                election.year
            );
        }
    }

    #[test]
    fn year_2026_uses_2024_as_the_explicit_proxy() {
        assert_eq!(baseline_election(2026).year, 2024);
        assert_eq!(baseline_election(1995).year, 1996);
    }

    #[test]
    fn event_windows_are_election_specific() {
        assert!(events_for_campaign(2016)
            .iter()
            .any(|event| event.year == 2014));
        assert!(!events_for_campaign(2020)
            .iter()
            .any(|event| event.year == 2020));
    }

    #[test]
    fn all_22_local_scenarios_are_selectable_with_common_tai_variant() {
        assert_eq!(REGIONS.len(), 22);
        assert_eq!(find_region("台北市").unwrap().key, "taipei");
        assert_eq!(find_region("Kinmen").unwrap().name, "金門縣");
    }
}
