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
