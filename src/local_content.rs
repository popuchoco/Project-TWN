use crate::game_content::Issue;

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
