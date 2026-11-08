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

/// Political-party profiles used for scenario affinities. The axes are game
/// parameters informed by public platforms and electoral alliances, not a
/// claim that parties have a single unchanging ideology.
#[derive(Clone, Copy, Debug)]
pub struct PartyProfile {
    pub name: &'static str,
    pub camp: &'static str,
    pub description: &'static str,
    /// -100 pan-Blue, +100 pan-Green; 0 means not reliably on either camp axis.
    pub blue_green_axis: i16,
    /// -100 left, +100 right.
    pub economic_axis: i16,
    /// -100 integration/unification-oriented, +100 Taiwan-sovereignty-oriented.
    pub cross_strait_axis: i16,
    /// Strength of religious or faith-community organizational networks.
    pub religious_network: u8,
}

pub const PARTY_PROFILES: &[PartyProfile] = &[
    PartyProfile {
        name: "中國國民黨",
        camp: "泛藍主流",
        description: "大型政黨；兩岸、經濟與治理議題的歷史主張隨時期變化。",
        blue_green_axis: -100,
        economic_axis: 15,
        cross_strait_axis: -20,
        religious_network: 5,
    },
    PartyProfile {
        name: "民主進步黨",
        camp: "泛綠主流",
        description: "大型政黨；以臺灣主體與民主治理為主要政治識別。",
        blue_green_axis: 100,
        economic_axis: -10,
        cross_strait_axis: 85,
        religious_network: 5,
    },
    PartyProfile {
        name: "台灣民眾黨",
        camp: "第三勢力／中間",
        description: "第三勢力；政治路線和合作策略依議題及時期而變。",
        blue_green_axis: 0,
        economic_axis: 0,
        cross_strait_axis: 5,
        religious_network: 0,
    },
    PartyProfile {
        name: "台灣團結聯盟",
        camp: "本土偏綠／右派取向",
        description: "臺灣本土與主權取向；此處依本次設計採用右派經濟軸與偏綠聯盟軸。",
        blue_green_axis: 75,
        economic_axis: 55,
        cross_strait_axis: 90,
        religious_network: 0,
    },
    PartyProfile {
        name: "親民黨",
        camp: "泛藍夥伴／中道自我定位",
        description: "曾與國民黨競爭、也曾合作；不能簡化成每次選舉都固定合流。",
        blue_green_axis: -65,
        economic_axis: 10,
        cross_strait_axis: -35,
        religious_network: 0,
    },
    PartyProfile {
        name: "新黨",
        camp: "泛藍／統派",
        description:
            "歷史上主張中國認同與兩岸統合；模型以兩岸路線表示，不用單一標籤代替其完整政見。",
        blue_green_axis: -90,
        economic_axis: 25,
        cross_strait_axis: -95,
        religious_network: 0,
    },
    PartyProfile {
        name: "時代力量",
        camp: "本土進步／中間偏左",
        description: "源於公民與社運參政；曾與民進黨合作，也自我定位為監督藍綠的第三勢力。",
        blue_green_axis: 55,
        economic_axis: -60,
        cross_strait_axis: 80,
        religious_network: 0,
    },
    PartyProfile {
        name: "社會民主黨",
        camp: "左派／社會民主",
        description: "以左派社會民主與多元民主為核心；曾與綠黨組成選舉聯盟，並非固定的大綠附屬。",
        blue_green_axis: 35,
        economic_axis: -90,
        cross_strait_axis: 55,
        religious_network: 0,
    },
    PartyProfile {
        name: "台灣綠黨",
        camp: "綠色政治／社會正義",
        description:
            "以生態智慧、社會正義、參與式民主與永續為核心；與泛綠合作是選舉情境，不作永久黨性設定。",
        blue_green_axis: 30,
        economic_axis: -70,
        cross_strait_axis: 35,
        religious_network: 0,
    },
    PartyProfile {
        name: "勞動黨",
        camp: "左翼／兩岸統合路線",
        description:
            "以社會主義及勞工議題為主，公開主張兩岸和解、逐步統一；不是泛藍或泛綠的簡單附屬。",
        blue_green_axis: 0,
        economic_axis: -95,
        cross_strait_axis: -95,
        religious_network: 0,
    },
    PartyProfile {
        name: "民國黨",
        camp: "歷史地方組織／宗教網絡",
        description: "歷史性政黨；本作以地方組織與宗教網絡作為遊戲特徵，不表示其2026年仍提名。",
        blue_green_axis: -5,
        economic_axis: 15,
        cross_strait_axis: -10,
        religious_network: 75,
    },
    PartyProfile {
        name: "國會政黨聯盟",
        camp: "歷史宗教動員型",
        description:
            "曾以宗教社群和傳統價值議題參與選舉；僅作歷史策略範本，不表示其 2026 年仍提名。",
        blue_green_axis: -10,
        economic_axis: 35,
        cross_strait_axis: -30,
        religious_network: 95,
    },
    PartyProfile {
        name: "綠黨社會民主黨聯盟",
        camp: "綠色／左派選舉聯盟",
        description: "2016年選舉合作名稱；採用聯盟路線，不把聯盟得票當作任一成員黨單獨得票。",
        blue_green_axis: 35,
        economic_axis: -80,
        cross_strait_axis: 45,
        religious_network: 0,
    },
];

/// Mutually-exclusive archetypes for the simulator's voter-movement model.
/// Their weights are adjustable game parameters, not measured population shares.
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
        name: "政黨認同／基本盤",
        weight_pct: 28.0,
        priority: Issue::Trust,
        responsiveness: 0.55,
        description: "較穩定；重視可信度、認同與投票動員。",
    },
    VoterSegment {
        name: "中間／游離選民",
        weight_pct: 22.0,
        priority: Issue::Governance,
        responsiveness: 1.35,
        description: "對治理評價、候選人表現及議題回應較敏感。",
    },
    VoterSegment {
        name: "青年／公民參與",
        weight_pct: 14.0,
        priority: Issue::Civic,
        responsiveness: 1.20,
        description: "以公民參與、教育、媒體與世代議題為關注入口。",
    },
    VoterSegment {
        name: "地方民生／生活成本",
        weight_pct: 16.0,
        priority: Issue::Local,
        responsiveness: 1.10,
        description: "在地服務、交通、產業、物價及照護等議題。",
    },
    VoterSegment {
        name: "兩岸／安全政策",
        weight_pct: 10.0,
        priority: Issue::CrossStrait,
        responsiveness: 1.15,
        description: "重視兩岸關係、國家安全與風險管理。",
    },
    VoterSegment {
        name: "平權／公共服務",
        weight_pct: 10.0,
        priority: Issue::Rights,
        responsiveness: 1.05,
        description: "平權、教育、公共衛生及社會安全網。",
    },
];

pub fn find_party(query: &str) -> Option<&'static PartyProfile> {
    let normalized = query.trim().replace('臺', "台");
    PARTY_PROFILES.iter().find(|party| {
        party.name.replace('臺', "台") == normalized
            || (normalized == "國民黨" && party.name == "中國國民黨")
            || (normalized == "民進黨" && party.name == "民主進步黨")
            || (normalized == "民眾黨" && party.name == "台灣民眾黨")
            || (normalized == "台聯" && party.name == "台灣團結聯盟")
            || (normalized == "綠黨" && party.name == "台灣綠黨")
            || (normalized == "國會政黨聯盟／民國黨"
                && (party.name == "國會政黨聯盟" || party.name == "民國黨"))
    })
}

/// A 0–100 scenario affinity based on the modeled party axes. This is not an
/// assertion of a permanent alliance; it is only a strategic cue for the
/// coalition mini-system.
pub fn party_affinity(left: &str, right: &str) -> f64 {
    let (Some(left), Some(right)) = (find_party(left), find_party(right)) else {
        return 45.0;
    };
    let distance = (left.blue_green_axis - right.blue_green_axis).abs() as f64 * 0.30
        + (left.economic_axis - right.economic_axis).abs() as f64 * 0.22
        + (left.cross_strait_axis - right.cross_strait_axis).abs() as f64 * 0.35
        + (left.religious_network as i16 - right.religious_network as i16).abs() as f64 * 0.13;
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
    /// Campaign years for which the card can be drawn. The event remains
    /// available as historical context outside this window.
    pub active_campaign_years: &'static [u16],
}

pub const EVENTS: &[EventCard] = &[
    EventCard {
        year: 1987,
        title: "解嚴後校園自治與黨政軍退出校園訴求",
        description: "解嚴後，學生自治、學術自由及要求軍訓教官退出校園等訴求逐步進入公共討論；相關制度變化延續多年，不能視為某次運動後即告完成。",
        issue: Issue::Education,
        active_campaign_years: &[1996, 2000, 2004],
    },
    EventCard {
        year: 1990,
        title: "野百合學運與國會／憲政改革",
        description: "三月學運要求解散萬年國會、廢除動員戡亂時期臨時條款並推動政治改革；運動促使政府召開國是會議，成為後續憲政改革的重要社會力量。",
        issue: Issue::Civic,
        active_campaign_years: &[1996, 2000, 2004],
    },
    EventCard {
        year: 1991,
        title: "終止動員戡亂與憲政轉型",
        description: "1991年5月1日終止動員戡亂時期並廢止《動員戡亂時期臨時條款》；其後國會改選、憲政改革與總統直選共同改變政治競爭規則。",
        issue: Issue::Governance,
        active_campaign_years: &[1996, 2000],
    },
    EventCard {
        year: 1993,
        title: "拉法葉艦採購與佣金疑雲",
        description: "臺灣於1991年簽約採購六艘拉法葉艦，後續圍繞佣金疑雲、海軍官兵命案、軍購監督與追討款項發展出長期調查及司法程序。此卡呈現採購透明與制度責任議題，不預判未經法院確定的個別指控。",
        issue: Issue::Governance,
        active_campaign_years: &[1996, 2000, 2004, 2008, 2012, 2016],
    },
    EventCard {
        year: 1992,
        title: "刑法第100條修正與言論自由",
        description: "1992年修正刑法第100條，普通內亂罪增列以強暴或脅迫為要件，單純政治言論不再因此成立該罪；「100行動聯盟」推動廢除政治刑法的社會運動背景亦值得理解。",
        issue: Issue::Rights,
        active_campaign_years: &[1996, 2000, 2004],
    },
    EventCard {
        year: 1995,
        title: "台海飛彈危機與首次總統直選",
        description: "1995至1996年軍事威懾與飛彈試射延續至首次總統直選前；模擬聚焦危機溝通、韌性與民主選擇，不預設民意必然向任何一方移動。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[1996, 2000],
    },
    EventCard {
        year: 1996,
        title: "首次總統直選",
        description: "1996年3月23日首次由選民直接選出正副總統，民主選舉成為全國性政治競爭的制度基礎。",
        issue: Issue::Civic,
        active_campaign_years: &[1996, 2000],
    },
    EventCard {
        year: 1998,
        title: "大學自治與校園軍訓制度轉型",
        description: "釋字第450號認定強制大學設置軍訓室的規定違反大學自治；教官職責調整及退出校園的爭論延續多年，教育部自2006年起讓各大學自主決定教官設置，呈現校園民主化及黨政軍影響逐步退場的制度過程。",
        issue: Issue::Education,
        active_campaign_years: &[2000, 2004, 2008, 2012, 2016],
    },
    EventCard {
        year: 1999,
        title: "兩岸定位與「特殊國與國關係」論述",
        description: "李登輝政府提出兩岸關係定位，引發國內外對主權、交流及風險管理的爭論。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2000, 2004],
    },
    EventCard {
        year: 2000,
        title: "首次和平政黨輪替與少數政府治理",
        description: "中央政權首次以和平方式移交，常被放在李登輝時期漸進民主化、俗稱「寧靜革命」的脈絡中理解；陳水扁政府未取得立法院過半，行政立法協商與政策落實成為執政難題。",
        issue: Issue::Governance,
        active_campaign_years: &[2004, 2008],
    },
    EventCard {
        year: 2000,
        title: "國防二法與軍隊國家化制度建構",
        description: "《國防法》及《國防部組織法》於2000年公布，相關國防組織改造於2002年3月1日啟動，強化文人領軍、國防軍政軍令整合與軍隊政治中立；這是制度建構的重要一步，而非民主轉型一夕完成。",
        issue: Issue::Governance,
        active_campaign_years: &[2004, 2008, 2012],
    },
    EventCard {
        year: 2002,
        title: "「一邊一國」論述與兩岸政策爭議",
        description: "陳水扁政府的兩岸表述引發支持者、在野黨與國際關係中的不同解讀；以立場溝通和危機管理作為競選議題。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2004, 2008],
    },
    EventCard {
        year: 2004,
        title: "三一九槍擊案與選後爭議",
        description: "投票前夕槍擊事件及其後的驗票、訴訟與社會爭議；模擬聚焦信任、資訊公開與降溫。",
        issue: Issue::Trust,
        active_campaign_years: &[2004, 2008],
    },
    EventCard {
        year: 2006,
        title: "施明德紅衫軍倒扁運動",
        description: "施明德於2006年發起百萬人民反貪腐倒扁運動，要求陳水扁總統下台；其後的政治信任危機是2008年政黨輪替的重要背景之一，但不能單獨視為敗選原因。",
        issue: Issue::Trust,
        active_campaign_years: &[2008, 2012],
    },
    EventCard {
        year: 2008,
        title: "陳雲林來台、警力維安與野草莓運動",
        description: "兩岸協商和大規模維安引發對集會自由及執法比例的批評；部分抗議者以「警察如公安」表達不滿，這是當時政治口號與抗議者觀點，並非本作的事實裁定。",
        issue: Issue::Civic,
        active_campaign_years: &[2012, 2016],
    },
    EventCard {
        year: 2010,
        title: "ECFA簽署與經濟整合爭論",
        description: "兩岸經濟合作架構協議於2010年簽署並生效；支持與批評聚焦市場機會、產業分配、依賴風險及程序監督。",
        issue: Issue::Economy,
        active_campaign_years: &[2012, 2016, 2020],
    },
    EventCard {
        year: 2012,
        title: "反媒體壟斷運動",
        description: "媒體集中與新聞自主成為公民團體和公共政策討論議題。",
        issue: Issue::Media,
        active_campaign_years: &[2016, 2020],
    },
    EventCard {
        year: 2012,
        title: "林益世收賄案與政治責任爭議",
        description: "2012年林益世案曝光後進入長期司法程序。案件不同部分的認定與結果並不相同：財產來源不明罪已判刑確定，索賄及洗錢部分無罪確定，收受約6300萬元部分曾多次發回更審；此卡聚焦廉政、權力關說與司法程序，不把各部分混作單一結果。",
        issue: Issue::Trust,
        active_campaign_years: &[2016, 2020, 2024, 2026],
    },
    EventCard {
        year: 2013,
        title: "洪仲丘事件與軍中人權改革",
        description: "義務役士兵洪仲丘死亡引發白衫軍等公民抗議，推動軍事審判制度改革與軍中人權、責任追究議題。",
        issue: Issue::Rights,
        active_campaign_years: &[2016, 2020],
    },
    EventCard {
        year: 2014,
        title: "太陽花學運",
        description: "學生與公民團體以服貿審查程序、國會監督及兩岸經貿為主要訴求。",
        issue: Issue::Civic,
        active_campaign_years: &[2014, 2016, 2020],
    },
    EventCard {
        year: 2014,
        title: "香港雨傘運動",
        description: "香港爭取普選的公民抗爭，成為臺灣社會討論民主制度、自治與兩岸政治框架的重要參照；不把香港事件直接等同臺灣選民態度。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2016, 2020, 2024],
    },
    EventCard {
        year: 2015,
        title: "反課綱微調運動",
        description: "高中課綱調整引發程序、史觀與教育參與的公共爭論。",
        issue: Issue::Education,
        active_campaign_years: &[2015, 2016, 2020],
    },
    EventCard {
        year: 2016,
        title: "周子瑜遭貼台獨標籤與道歉影片爭議",
        description: "總統投票前夕，藝人黃安在社群平台指稱周子瑜為台獨，周子瑜其後發布道歉影片並暫停中國大陸活動，引發政治標籤、跨境市場壓力、未成年人處境及言論自主的討論。此事件與愛國同心會相關議題分開呈現；研究可討論其選舉傳播背景，但沒有足夠證據將固定票數效果歸因於單一事件。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2016, 2020, 2024, 2026],
    },
    EventCard {
        year: 2016,
        title: "愛國同心會與香港人士討論港獨衝突",
        description: "2016年臺北一處飯店發生愛國同心會人士因鄰座香港人士談論港獨而上前嗆聲、演變成肢體衝突的事件，警方表示將相關人士依法處理。此卡聚焦異議表達、政治標籤與公共安全；與黃安對周子瑜的社群貼文及其後的選舉傳播分開記錄。",
        issue: Issue::Civic,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2016,
        title: "民進黨首次完全執政的施政責任",
        description: "民進黨首次同時取得總統職位與立法院多數；政策推動、改革速度、利益分配與監督成為支持者和反對者檢視焦點。",
        issue: Issue::Governance,
        active_campaign_years: &[2020, 2024],
    },
    EventCard {
        year: 2017,
        title: "轉型正義與政治檔案開放",
        description: "促進轉型正義、政治檔案徵集與解密公開逐步制度化；討論涉及歷史真相、責任、社會和解及個人資料權益。",
        issue: Issue::Trust,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2016,
        title: "兩岸政策與維持現狀的政策辯論",
        description: "蔡英文政府以憲法及兩岸人民關係條例等處理兩岸事務，並強調維持現狀；各政黨對其政治基礎、對話條件及風險管理有不同主張。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2020, 2024],
    },
    EventCard {
        year: 2018,
        title: "地方選舉與「韓流」現象",
        description: "2018年地方選舉中，韓國瑜在高雄勝選並帶動全國政治討論。研究將其傳播、訴求與支持擴張列為觀察面向；本作不把它設定成任何政黨的固定加成。",
        issue: Issue::Economy,
        active_campaign_years: &[2020, 2024],
    },
    EventCard {
        year: 2018,
        title: "李新死因影片爭議與羅淑蕾名譽訴訟",
        description: "前臺北市議員李新去世後，相關影片提出涉及羅淑蕾等人的指控；羅淑蕾聲請移除影片的假處分遭法院駁回。該程序裁定不是對影片指控真偽的判決；本卡呈現政治人物名譽、媒體傳播與司法救濟的爭論。",
        issue: Issue::Media,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2018,
        title: "愛國同心會成員參選與賄選判決",
        description: "愛國同心會成員張秀葉參選2018年臺北市議員選舉，涉向選民提供餐飲。最高法院於2022年駁回上訴、判刑3年5月定讞。此為特定候選人及個案的司法結果，與黃安對周子瑜的社群貼文不是同一事件，也不據此推論所有組織成員或政治檢舉皆違法。",
        issue: Issue::Trust,
        active_campaign_years: &[2022, 2024, 2026],
    },
    EventCard {
        year: 2019,
        title: "同性婚姻法制化",
        description: "司法院釋字第748號解釋施行法於2019年5月24日施行；社會平權與價值政策進入制度實踐。",
        issue: Issue::Rights,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2019,
        title: "香港反送中運動",
        description: "香港逃犯條例修訂引發大規模示威及持續爭議，臺灣社會關注自治、人權與「一國兩制」承諾；不預設對特定臺灣政黨的投票效果。",
        issue: Issue::Civic,
        active_campaign_years: &[2020, 2024, 2026],
    },
    EventCard {
        year: 2019,
        title: "反滲透法立法與民主防衛辯論",
        description: "反滲透法於2020年1月公布施行；防制境外敵對勢力干預與政治活動界線、法定要件及公民自由保障成為公共辯論。2020總統投票早於公布時間。",
        issue: Issue::Governance,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2020,
        title: "香港國安法實施",
        description: "香港國安法於2020年6月實施，臺灣社會關注香港自治、公民自由、法治及兩岸政治制度差異。",
        issue: Issue::Rights,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2020,
        title: "COVID-19疫情治理",
        description: "臺灣於2020年1月21日公布首例境外移入確診；該日期晚於1月11日總統投票日，因此不作為2020總統票的選前因素，供後續選戰檢視治理經驗。",
        issue: Issue::PublicHealth,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2021,
        title: "彭文正遭通緝與言論／司法程序爭議",
        description: "彭文正因涉加重誹謗蔡英文學歷而遭起訴，未到庭後臺北地方法院於2021年發布通緝。此卡記錄該刑事案件的程序狀態，不把通緝視為有罪判決；其另行提出、涉及蔡英文論文的自訴案件於2026年最高法院駁回確定，屬不同案件及程序。",
        issue: Issue::Rights,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2022,
        title: "地方選舉的執政評價",
        description: "地方選舉結果提醒參選者：地方服務、候選人特質及中央執政評價可能彼此不同，勿將地方票直接等同全國民意。",
        issue: Issue::Local,
        active_campaign_years: &[2024, 2026],
    },
    EventCard {
        year: 2024,
        title: "香港基本法第23條立法",
        description: "香港《維護國家安全條例》於2024年3月生效；臺灣公共討論涉及國安治理、自治、人權及國際信任。",
        issue: Issue::Rights,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2024,
        title: "柯文哲京華城與政治獻金案件訴訟",
        description: "柯文哲涉京華城及政治獻金等案於2024年遭起訴；臺北地方法院2026年3月一審判決後，柯文哲提起上訴，高等法院於2026年9月開始審理政治獻金部分。截至2026年11月8日尚未終局確定，故分清起訴、一審與上訴程序，不以未確定判決預設政治或選舉效果。",
        issue: Issue::Trust,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2024,
        title: "賴政府兩岸政策與區域安全",
        description: "新政府宣示依憲政體制處理兩岸事務，並以和平、對等、民主、對話作原則；兩岸對話、主權表述與安全風險管理持續受到檢視。",
        issue: Issue::CrossStrait,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2024,
        title: "三黨不過半的國會",
        description: "立法院席次由國民黨52、民進黨51、民眾黨8席及無黨籍2席組成；協商、監督與政策溝通成為執政議題。",
        issue: Issue::Governance,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2024,
        title: "政治檔案條例修正與轉型正義",
        description: "政治檔案條例修正於2024年2月28日施行，涉及檔案徵集、解密公開、歷史真相與當事人權益平衡。",
        issue: Issue::Trust,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2025,
        title: "大規模立委罷免投票與國會僵局",
        description: "2025年兩輪罷免投票均未通過，顯示高強度動員不必然轉化為制度性席次改變；動員、反動員、地方盤面與法定門檻都會影響結果。",
        issue: Issue::Civic,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2026,
        title: "國安法與反滲透法修法提案",
        description: "2026年出現多項涉及跨境鎮壓、境外指示與國安法制的修法提案；截至11月8日仍屬提案與審議中的政治議題，並非已完成的新法。",
        issue: Issue::Governance,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2026,
        title: "香港國安法第43條實施細則修訂",
        description: "香港特區政府於2026年3月23日公布並即日實施第43條實施細則修訂；臺灣社會持續討論香港法治、人權與區域制度差異。",
        issue: Issue::Rights,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2026,
        title: "「洋流」選戰敘事與地方參與",
        description: "2026年媒體與競選活動以「洋流」討論青年參與、地方互動及支持擴散；其政治意義仍有不同解讀。本作將其作為可回應的公民參與議題，不預設特定陣營受益。",
        issue: Issue::Civic,
        active_campaign_years: &[2026],
    },
    EventCard {
        year: 2026,
        title: "地方公職人員選舉（尚未投票）",
        description: "中選會訂定投票日為2026年11月28日；截至2026年11月8日結果尚未產生。本作只提供情境模擬，不預填勝負。",
        issue: Issue::Local,
        active_campaign_years: &[2026],
    },
];

#[derive(Clone, Copy, Debug)]
pub struct LegislatureSnapshot {
    pub year: u16,
    pub summary: &'static str,
}

pub const LEGISLATURE_SNAPSHOTS: &[LegislatureSnapshot] = &[
    LegislatureSnapshot {
        year: 1998,
        summary: "國民黨123席、民進黨70席、新黨11席、其他／無黨籍21席；新黨仍是能改變席次協商的第三黨。",
    },
    LegislatureSnapshot {
        year: 2001,
        summary: "民進黨87席、國民黨68席、親民黨46席、台聯13席、新黨1席、其他／無黨籍10席；多個第三勢力影響朝野席次。",
    },
    LegislatureSnapshot {
        year: 2004,
        summary: "民進黨89席、國民黨79席、親民黨34席、台聯12席、新黨1席、無黨團結聯盟6席、其他／無黨籍4席；兩大陣營均依賴友黨協商。",
    },
    LegislatureSnapshot {
        year: 2008,
        summary: "國民黨81席、民進黨27席、親民黨1席、其他／無黨籍4席；政黨輪替伴隨小黨席次重整。",
    },
    LegislatureSnapshot {
        year: 2012,
        summary: "國民黨64席、民進黨40席、親民黨3席、台聯3席、其他／無黨籍3席；小黨政黨票改變兩大黨票源。",
    },
    LegislatureSnapshot {
        year: 2016,
        summary: "民進黨68席、國民黨35席、時代力量5席、親民黨3席、其他2席；民進黨首度完全執政，時力進入國會。",
    },
    LegislatureSnapshot {
        year: 2020,
        summary: "民進黨61席、國民黨38席、民眾黨5席、時代力量3席、台灣基進1席、無黨籍5席。",
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
    fn political_event_cards_keep_distinct_legal_and_historical_contexts() {
        let titles: Vec<_> = EVENTS.iter().map(|event| event.title).collect();
        for expected in [
            "拉法葉艦採購與佣金疑雲",
            "林益世收賄案與政治責任爭議",
            "周子瑜遭貼台獨標籤與道歉影片爭議",
            "愛國同心會與香港人士討論港獨衝突",
            "李新死因影片爭議與羅淑蕾名譽訴訟",
            "愛國同心會成員參選與賄選判決",
            "彭文正遭通緝與言論／司法程序爭議",
            "柯文哲京華城與政治獻金案件訴訟",
        ] {
            assert!(titles.contains(&expected), "missing event card: {expected}");
        }

        let chou = EVENTS
            .iter()
            .find(|event| event.title.contains("周子瑜"))
            .unwrap();
        let patriot_association = EVENTS
            .iter()
            .find(|event| event.title == "愛國同心會成員參選與賄選判決")
            .unwrap();
        assert_ne!(chou.title, patriot_association.title);
        assert!(chou.description.contains("與愛國同心會相關議題分開呈現"));
        assert!(patriot_association.description.contains("特定候選人及個案"));
    }

    #[test]
    fn all_22_local_scenarios_are_selectable_with_common_tai_variant() {
        assert_eq!(REGIONS.len(), 22);
        assert_eq!(find_region("台北市").unwrap().key, "taipei");
        assert_eq!(find_region("Kinmen").unwrap().name, "金門縣");
    }
}
