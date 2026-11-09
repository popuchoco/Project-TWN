//! Fictional election, party, region, and event content for playable builds.
//!
//! Numeric values are scenario parameters. They do not report or reproduce
//! real election results, political organizations, people, or places.

#[derive(Clone, Copy, Debug)]
pub struct ScenarioCandidate {
    pub president: &'static str,
    pub vice_president: &'static str,
    pub party: &'static str,
    pub votes: u64,
    pub share_pct: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct ElectionScenario {
    pub year: u16,
    pub date: &'static str,
    pub incumbent_party: &'static str,
    pub candidates: &'static [ScenarioCandidate],
}

pub const ELECTIONS: &[ElectionScenario] = &[
    ElectionScenario {
        year: 1996,
        date: "1996-10-26",
        incumbent_party: "群島共治黨",
        candidates: &[
            ScenarioCandidate {
                president: "岑以川",
                vice_president: "葉承序",
                party: "群島共治黨",
                votes: 561_000,
                share_pct: 51.0,
            },
            ScenarioCandidate {
                president: "沈以霏",
                vice_president: "陳書安",
                party: "海岳協作黨",
                votes: 374_000,
                share_pct: 34.0,
            },
            ScenarioCandidate {
                president: "姜映寧",
                vice_president: "江予琛",
                party: "曙光民主聯盟",
                votes: 165_000,
                share_pct: 15.0,
            },
        ],
    },
    ElectionScenario {
        year: 2000,
        date: "2000-10-21",
        incumbent_party: "海岳協作黨",
        candidates: &[
            ScenarioCandidate {
                president: "蘇映泉",
                vice_president: "葉嘉庭",
                party: "曙光民主聯盟",
                votes: 471_500,
                share_pct: 41.0,
            },
            ScenarioCandidate {
                president: "林漱川",
                vice_president: "沈亦安",
                party: "海岳協作黨",
                votes: 437_000,
                share_pct: 38.0,
            },
            ScenarioCandidate {
                president: "方景行",
                vice_president: "江清禾",
                party: "群島共治黨",
                votes: 241_500,
                share_pct: 21.0,
            },
        ],
    },
    ElectionScenario {
        year: 2004,
        date: "2004-10-16",
        incumbent_party: "曙光民主聯盟",
        candidates: &[
            ScenarioCandidate {
                president: "簡雨川",
                vice_president: "邱若衡",
                party: "曙光民主聯盟",
                votes: 640_000,
                share_pct: 51.2,
            },
            ScenarioCandidate {
                president: "周望舒",
                vice_president: "許景堯",
                party: "群島共治黨",
                votes: 610_000,
                share_pct: 48.8,
            },
        ],
    },
    ElectionScenario {
        year: 2008,
        date: "2008-10-11",
        incumbent_party: "曙光民主聯盟",
        candidates: &[
            ScenarioCandidate {
                president: "穆以安",
                vice_president: "范明澤",
                party: "群島共治黨",
                votes: 672_000,
                share_pct: 56.0,
            },
            ScenarioCandidate {
                president: "顧知遠",
                vice_president: "白映棠",
                party: "曙光民主聯盟",
                votes: 528_000,
                share_pct: 44.0,
            },
        ],
    },
    ElectionScenario {
        year: 2012,
        date: "2012-10-20",
        incumbent_party: "群島共治黨",
        candidates: &[
            ScenarioCandidate {
                president: "唐予澄",
                vice_president: "葉致寧",
                party: "群島共治黨",
                votes: 637_000,
                share_pct: 49.0,
            },
            ScenarioCandidate {
                president: "林朝雨",
                vice_president: "韓立安",
                party: "曙光民主聯盟",
                votes: 598_000,
                share_pct: 46.0,
            },
            ScenarioCandidate {
                president: "游以禾",
                vice_president: "許南風",
                party: "地方共好聯盟",
                votes: 65_000,
                share_pct: 5.0,
            },
        ],
    },
    ElectionScenario {
        year: 2016,
        date: "2016-10-15",
        incumbent_party: "群島共治黨",
        candidates: &[
            ScenarioCandidate {
                president: "夏允舟",
                vice_president: "江之衡",
                party: "曙光民主聯盟",
                votes: 715_500,
                share_pct: 53.0,
            },
            ScenarioCandidate {
                president: "沈既明",
                vice_president: "葉知夏",
                party: "群島共治黨",
                votes: 472_500,
                share_pct: 35.0,
            },
            ScenarioCandidate {
                president: "方映川",
                vice_president: "楊千薇",
                party: "公民新聲黨",
                votes: 162_000,
                share_pct: 12.0,
            },
        ],
    },
    ElectionScenario {
        year: 2020,
        date: "2020-10-17",
        incumbent_party: "曙光民主聯盟",
        candidates: &[
            ScenarioCandidate {
                president: "葉承遠",
                vice_president: "顧予晴",
                party: "曙光民主聯盟",
                votes: 770_000,
                share_pct: 55.0,
            },
            ScenarioCandidate {
                president: "江景岳",
                vice_president: "周念慈",
                party: "群島共治黨",
                votes: 546_000,
                share_pct: 39.0,
            },
            ScenarioCandidate {
                president: "白千嶼",
                vice_president: "陳禾安",
                party: "地方共好聯盟",
                votes: 84_000,
                share_pct: 6.0,
            },
        ],
    },
    ElectionScenario {
        year: 2024,
        date: "2024-10-12",
        incumbent_party: "曙光民主聯盟",
        candidates: &[
            ScenarioCandidate {
                president: "林沐青",
                vice_president: "江予禾",
                party: "曙光民主聯盟",
                votes: 609_000,
                share_pct: 42.0,
            },
            ScenarioCandidate {
                president: "沈知行",
                vice_president: "葉以棠",
                party: "群島共治黨",
                votes: 507_500,
                share_pct: 35.0,
            },
            ScenarioCandidate {
                president: "方知夏",
                vice_president: "顧星野",
                party: "公民新聲黨",
                votes: 333_500,
                share_pct: 23.0,
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
        key: "haze-city",
        name: "霧川市",
        previous_winner_party: "群島共治黨",
        focus: Issue::Local,
        focus_label: "居住、交通與城市治理",
    },
    RegionProfile {
        key: "starfield-city",
        name: "星野市",
        previous_winner_party: "海岳協作黨",
        focus: Issue::Governance,
        focus_label: "公共服務與城市更新",
    },
    RegionProfile {
        key: "bluecape-city",
        name: "青岬市",
        previous_winner_party: "曙光民主聯盟",
        focus: Issue::Economy,
        focus_label: "產業轉型與通勤交通",
    },
    RegionProfile {
        key: "hope-isles-city",
        name: "望洲市",
        previous_winner_party: "地方共好聯盟",
        focus: Issue::Local,
        focus_label: "港區、觀光與漁業",
    },
    RegionProfile {
        key: "warmridge-city",
        name: "懷嶺市",
        previous_winner_party: "群島共治黨",
        focus: Issue::PublicHealth,
        focus_label: "醫療可近性與長照",
    },
    RegionProfile {
        key: "cloudrest-city",
        name: "雲棲市",
        previous_winner_party: "海岳協作黨",
        focus: Issue::Education,
        focus_label: "校園與青年公共服務",
    },
    RegionProfile {
        key: "southstream-county",
        name: "南澗縣",
        previous_winner_party: "曙光民主聯盟",
        focus: Issue::Economy,
        focus_label: "農業、觀光與財政",
    },
    RegionProfile {
        key: "northbay-county",
        name: "北浦縣",
        previous_winner_party: "群島共治黨",
        focus: Issue::Local,
        focus_label: "山城交通與防災",
    },
    RegionProfile {
        key: "teasource-county",
        name: "茶源縣",
        previous_winner_party: "地方共好聯盟",
        focus: Issue::Economy,
        focus_label: "農業、產業與空品",
    },
    RegionProfile {
        key: "duskysea-county",
        name: "暮海縣",
        previous_winner_party: "海岳協作黨",
        focus: Issue::Local,
        focus_label: "沿岸建設與交通",
    },
    RegionProfile {
        key: "linkedplain-county",
        name: "連原縣",
        previous_winner_party: "群島共治黨",
        focus: Issue::Economy,
        focus_label: "產業、人口與公共運輸",
    },
    RegionProfile {
        key: "islelight-county",
        name: "島嶼縣",
        previous_winner_party: "曙光民主聯盟",
        focus: Issue::Local,
        focus_label: "離島交通、觀光與照護",
    },
    RegionProfile {
        key: "farhill-county",
        name: "遠山縣",
        previous_winner_party: "地方共好聯盟",
        focus: Issue::Local,
        focus_label: "山區通訊與災後韌性",
    },
    RegionProfile {
        key: "bathstone-county",
        name: "沐石縣",
        previous_winner_party: "海岳協作黨",
        focus: Issue::Education,
        focus_label: "校園整修與教育資源",
    },
    RegionProfile {
        key: "orangebay-county",
        name: "橘灣縣",
        previous_winner_party: "群島共治黨",
        focus: Issue::Economy,
        focus_label: "港灣產業與勞動權益",
    },
    RegionProfile {
        key: "greenvale-county",
        name: "翠谷縣",
        previous_winner_party: "曙光民主聯盟",
        focus: Issue::Rights,
        focus_label: "社區照護與平等服務",
    },
    RegionProfile {
        key: "lakeplain-county",
        name: "湖原縣",
        previous_winner_party: "海岳協作黨",
        focus: Issue::Local,
        focus_label: "觀光、交通與環境",
    },
    RegionProfile {
        key: "reedshore-county",
        name: "蘆汀縣",
        previous_winner_party: "地方共好聯盟",
        focus: Issue::Economy,
        focus_label: "農產運輸與青年返鄉",
    },
    RegionProfile {
        key: "glowcoast-county",
        name: "沅光縣",
        previous_winner_party: "群島共治黨",
        focus: Issue::Local,
        focus_label: "公共建設與社區安全",
    },
    RegionProfile {
        key: "jadeport-county",
        name: "碧港縣",
        previous_winner_party: "曙光民主聯盟",
        focus: Issue::Governance,
        focus_label: "港口治理與採購透明",
    },
    RegionProfile {
        key: "tideisle-county",
        name: "潮嶼縣",
        previous_winner_party: "海岳協作黨",
        focus: Issue::CrossStrait,
        focus_label: "島際交通與區域合作",
    },
    RegionProfile {
        key: "clearsea-county",
        name: "沁海縣",
        previous_winner_party: "地方共好聯盟",
        focus: Issue::CrossStrait,
        focus_label: "離島建設與對外往來",
    },
];

pub fn find_region(query: &str) -> Option<&'static RegionProfile> {
    let normalized = normalize_region_name(query);
    REGIONS.iter().find(|region| {
        region.key.eq_ignore_ascii_case(query.trim())
            || normalize_region_name(region.name) == normalized
    })
}

#[derive(Clone, Copy, Debug)]
pub struct PartyProfile {
    pub name: &'static str,
    pub camp: &'static str,
    pub description: &'static str,
    pub governance_axis: i16,
    pub economic_axis: i16,
    pub regional_relations_axis: i16,
    pub community_network: u8,
}

pub const PARTY_PROFILES: &[PartyProfile] = &[
    PartyProfile {
        name: "群島共治黨",
        camp: "大型治理聯盟",
        description: "重視行政連續與跨區協調；不同議題仍可能出現黨內歧見。",
        governance_axis: -65,
        economic_axis: 20,
        regional_relations_axis: 15,
        community_network: 30,
    },
    PartyProfile {
        name: "海岳協作黨",
        camp: "地方組織聯盟",
        description: "以地方網絡與產業協商為強項；政策取向依候選人與地區而變。",
        governance_axis: -20,
        economic_axis: 45,
        regional_relations_axis: -15,
        community_network: 65,
    },
    PartyProfile {
        name: "曙光民主聯盟",
        camp: "改革派聯盟",
        description: "重視公共參與、制度改革與公民權利；施政承諾仍須面對資源取捨。",
        governance_axis: 70,
        economic_axis: -15,
        regional_relations_axis: 55,
        community_network: 35,
    },
    PartyProfile {
        name: "地方共好聯盟",
        camp: "跨區地方聯盟",
        description: "以地方服務、跨區協作和自治議題為主；不預設固定全國路線。",
        governance_axis: 10,
        economic_axis: 5,
        regional_relations_axis: 10,
        community_network: 80,
    },
    PartyProfile {
        name: "公民新聲黨",
        camp: "新興中間團體",
        description: "由地方倡議者組成，重視公開資訊與新型公共溝通。",
        governance_axis: 35,
        economic_axis: -5,
        regional_relations_axis: 25,
        community_network: 25,
    },
    PartyProfile {
        name: "社會同行黨",
        camp: "社會公平派",
        description: "優先處理勞動條件、社會安全網與公共服務可近性。",
        governance_axis: 30,
        economic_axis: -75,
        regional_relations_axis: 10,
        community_network: 40,
    },
    PartyProfile {
        name: "信望共益聯盟",
        camp: "社區信念網絡",
        description: "以社區互助與家庭服務網絡為特色；信仰不代表成員立場一致。",
        governance_axis: -5,
        economic_axis: 20,
        regional_relations_axis: -10,
        community_network: 90,
    },
    PartyProfile {
        name: "永續綠境黨",
        camp: "環境永續派",
        description: "以環境承載、能源轉型與地方韌性作為政策優先項。",
        governance_axis: 45,
        economic_axis: -45,
        regional_relations_axis: 35,
        community_network: 25,
    },
    PartyProfile {
        name: "勞動新路黨",
        camp: "勞動權益派",
        description: "將工時、薪資、安全與產業轉型納入地方競選主軸。",
        governance_axis: 15,
        economic_axis: -85,
        regional_relations_axis: -20,
        community_network: 35,
    },
    PartyProfile {
        name: "新契約聯盟",
        camp: "制度更新派",
        description: "主張政府資訊公開、服務流程檢討與地方治理實驗。",
        governance_axis: 55,
        economic_axis: 10,
        regional_relations_axis: 5,
        community_network: 20,
    },
    PartyProfile {
        name: "山海自治黨",
        camp: "區域自治派",
        description: "重視島際合作、山海交通與地方議會參與。",
        governance_axis: 0,
        economic_axis: -5,
        regional_relations_axis: -45,
        community_network: 60,
    },
    PartyProfile {
        name: "傳統價值聯盟",
        camp: "社區傳承派",
        description: "重視文化保存與代間互助，內部對政策手段保有多元看法。",
        governance_axis: -30,
        economic_axis: 35,
        regional_relations_axis: -25,
        community_network: 75,
    },
];

pub fn find_party(query: &str) -> Option<&'static PartyProfile> {
    let normalized = query.trim();
    PARTY_PROFILES.iter().find(|party| {
        party.name == normalized
            || (normalized == "群島黨" && party.name == "群島共治黨")
            || (normalized == "地方聯盟" && party.name == "地方共好聯盟")
            || (normalized == "綠境黨" && party.name == "永續綠境黨")
    })
}

pub fn party_affinity(left: &str, right: &str) -> f64 {
    let (Some(left), Some(right)) = (find_party(left), find_party(right)) else {
        return 45.0;
    };
    let distance = (left.governance_axis - right.governance_axis).abs() as f64 * 0.30
        + (left.economic_axis - right.economic_axis).abs() as f64 * 0.22
        + (left.regional_relations_axis - right.regional_relations_axis).abs() as f64 * 0.35
        + (left.community_network as i16 - right.community_network as i16).abs() as f64 * 0.13;
    (100.0 - distance / 2.0).clamp(0.0, 100.0)
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
