//! A short, choice-driven campaign-adviser mode.
//!
//! Characters and situations are fictional composites. Historical event cards
//! can contextualize a briefing, but they do not determine a party's benefit.

use crate::game_content::Issue;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignFocus {
    Livelihood,
    OpenGovernment,
    DemocraticResilience,
}

impl CampaignFocus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Livelihood => "民生與生活成本",
            Self::OpenGovernment => "誠信與透明治理",
            Self::DemocraticResilience => "民主韌性與權利保障",
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Livelihood => "livelihood",
            Self::OpenGovernment => "open_government",
            Self::DemocraticResilience => "democratic_resilience",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StafferChoiceId {
    FocusLivelihood,
    FocusOpenGovernment,
    FocusDemocracy,
    PetitionFieldVisit,
    PetitionVerify,
    PetitionLocalLiaison,
    NetworkListening,
    NetworkForum,
    NetworkCoordination,
    EventVerify,
    EventService,
    EventCounter,
    PollReframe,
    PollGroundwork,
    PollHoldLine,
    ClosingDebate,
    ClosingField,
    ClosingCoalition,
}

impl StafferChoiceId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::FocusLivelihood => "focus_livelihood",
            Self::FocusOpenGovernment => "focus_open_government",
            Self::FocusDemocracy => "focus_democracy",
            Self::PetitionFieldVisit => "petition_field_visit",
            Self::PetitionVerify => "petition_verify",
            Self::PetitionLocalLiaison => "petition_local_liaison",
            Self::NetworkListening => "network_listening",
            Self::NetworkForum => "network_forum",
            Self::NetworkCoordination => "network_coordination",
            Self::EventVerify => "event_verify",
            Self::EventService => "event_service",
            Self::EventCounter => "event_counter",
            Self::PollReframe => "poll_reframe",
            Self::PollGroundwork => "poll_groundwork",
            Self::PollHoldLine => "poll_hold_line",
            Self::ClosingDebate => "closing_debate",
            Self::ClosingField => "closing_field",
            Self::ClosingCoalition => "closing_coalition",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StafferDecision {
    Chosen(StafferChoiceId),
    Skipped,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StafferProgress {
    decisions: [Option<StafferDecision>; 6],
    focus: Option<CampaignFocus>,
}

impl StafferProgress {
    pub const fn focus(self) -> Option<CampaignFocus> {
        self.focus
    }

    pub fn completed_briefings(self) -> u8 {
        self.decisions
            .iter()
            .filter(|decision| decision.is_some())
            .count() as u8
    }

    pub fn ending_note(self) -> &'static str {
        match (self.focus, self.decision(5)) {
            (
                Some(CampaignFocus::Livelihood),
                Some(StafferDecision::Chosen(StafferChoiceId::ClosingField)),
            ) => "最後一週，團隊把主軸交給地方行程；志工交班後，林予安把未完成的居民回覆清單留在桌上，提醒大家選舉結束後仍要有人接手。",
            (
                Some(CampaignFocus::OpenGovernment),
                Some(StafferDecision::Chosen(StafferChoiceId::ClosingDebate)),
            ) => "最後一場說明列出成本、期限與尚未完成的項目。候選人沒有把所有問題說成已解決，團隊也知道這種誠實未必換來掌聲。",
            (
                Some(CampaignFocus::DemocraticResilience),
                Some(StafferDecision::Chosen(StafferChoiceId::ClosingCoalition)),
            ) => "共同政見與合作界線都已公開；夥伴各自走向最後行程，帶走的是一份共同承諾，也保留了彼此不同的主張。",
            (_, Some(StafferDecision::Skipped)) => {
                "最後布局沒有另作幕僚會議決策，團隊依每日行動走到投票日前夕；最終結果仍由整場戰役的選擇與模型結算決定。"
            }
            _ => "戰役走到投票日前夕，沒有任何一項單獨選擇保證勝負；團隊只能對一路做過的承諾與取捨負責。",
        }
    }

    pub fn summary(self) -> String {
        if self.decisions.iter().all(Option::is_none) {
            return "-".to_string();
        }
        let mut route = Vec::with_capacity(self.decisions.len() + 1);
        route.push(self.focus.map(CampaignFocus::key).unwrap_or("no_focus"));
        route.extend(self.decisions.iter().map(|decision| match decision {
            Some(StafferDecision::Chosen(id)) => id.key(),
            Some(StafferDecision::Skipped) => "skip",
            None => "pending",
        }));
        route.join(">")
    }

    pub(crate) fn decision(self, index: usize) -> Option<StafferDecision> {
        self.decisions.get(index).copied().flatten()
    }

    pub(crate) fn record(
        &mut self,
        index: usize,
        decision: StafferDecision,
        focus: Option<CampaignFocus>,
    ) -> bool {
        let Some(slot) = self.decisions.get_mut(index) else {
            return false;
        };
        if slot.is_some() {
            return false;
        }
        *slot = Some(decision);
        if index == 0 {
            self.focus = focus;
        }
        true
    }

    pub(crate) fn is_valid(self, round: u8) -> bool {
        for (index, decision) in self.decisions.iter().enumerate() {
            if decision.is_some() && round < (index as u8 * 2) {
                return false;
            }
            if let Some(StafferDecision::Chosen(id)) = decision {
                if choice_for(index, *id).is_none() {
                    return false;
                }
            }
        }
        match self.decisions[0] {
            Some(StafferDecision::Chosen(id)) => {
                self.focus == choice_for(0, id).and_then(|choice| choice.focus)
            }
            Some(StafferDecision::Skipped) | None => self.focus.is_none(),
        }
    }

    fn aligned_choices_before(self, index: usize) -> usize {
        let Some(focus) = self.focus else {
            return 0;
        };
        self.decisions
            .iter()
            .take(index)
            .enumerate()
            .filter(|(stage, decision)| {
                let Some(StafferDecision::Chosen(id)) = decision else {
                    return false;
                };
                choice_for(*stage, *id).is_some_and(|choice| choice.affinity == Some(focus))
            })
            .count()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StafferChoiceSpec {
    pub id: StafferChoiceId,
    pub label: &'static str,
    pub description: &'static str,
    pub budget_cost: i32,
    pub support_delta: f64,
    pub trust_delta: f64,
    pub mobilization_delta: f64,
    pub stamina_delta: f64,
    pub issue: Issue,
    pub affinity: Option<CampaignFocus>,
    pub focus: Option<CampaignFocus>,
}

#[derive(Clone, Copy, Debug)]
pub struct StafferBrief {
    pub index: usize,
    pub week: u8,
    pub title: &'static str,
    pub character: &'static str,
    pub scene: &'static str,
    pub choices: &'static [StafferChoiceSpec],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StafferBonus {
    pub support_delta: f64,
    pub trust_delta: f64,
    pub note: Option<&'static str>,
}

const FOCUS_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::FocusLivelihood,
        label: "以生活成本、交通與地方服務作為主軸",
        description: "先處理選民每天碰到的問題；需要安排地方行程與政策成本說明。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 0.6,
        mobilization_delta: 0.5,
        stamina_delta: -0.5,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::Livelihood),
        focus: Some(CampaignFocus::Livelihood),
    },
    StafferChoiceSpec {
        id: StafferChoiceId::FocusOpenGovernment,
        label: "以誠信、資訊公開與可檢驗治理作為主軸",
        description: "建立承諾追蹤方式；短期聲量未必最高，但能累積團隊信任。",
        budget_cost: 1,
        support_delta: 0.04,
        trust_delta: 1.6,
        mobilization_delta: 0.1,
        stamina_delta: -0.4,
        issue: Issue::Governance,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: Some(CampaignFocus::OpenGovernment),
    },
    StafferChoiceSpec {
        id: StafferChoiceId::FocusDemocracy,
        label: "以民主韌性、公民參與與權利保障作為主軸",
        description: "連結公共參與與民主議題；團隊需持續用具體政策說明支撐主張。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 0.8,
        mobilization_delta: 0.8,
        stamina_delta: -0.6,
        issue: Issue::Civic,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: Some(CampaignFocus::DemocraticResilience),
    },
];

const PETITION_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::PetitionFieldVisit,
        label: "安排現勘並建立居民回訪清單",
        description: "花費較多經費與體力，換取第一手需求及後續追蹤基礎。",
        budget_cost: 2,
        support_delta: 0.07,
        trust_delta: 1.2,
        mobilization_delta: 0.7,
        stamina_delta: -2.0,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::Livelihood),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::PetitionVerify,
        label: "先查證權責與資料，再公開說明已知和待查事項",
        description: "讓資訊與承諾可核對；回覆較慢，但能降低誤傳與過度承諾。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 1.8,
        mobilization_delta: 0.1,
        stamina_delta: -1.0,
        issue: Issue::Governance,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::PetitionLocalLiaison,
        label: "拜會里辦公處與民間團體，確認聯絡窗口",
        description: "串起地方網絡、釐清陳情流程；需要協調時間，不能保證立即解決。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 0.8,
        mobilization_delta: 0.9,
        stamina_delta: -1.5,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: None,
    },
];

const NETWORK_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::NetworkListening,
        label: "拜訪地方組織與社區團體，先聽需求再談分工",
        description: "有助建立長期協作，但行程密集、短期動員較有限。",
        budget_cost: 2,
        support_delta: 0.06,
        trust_delta: 1.0,
        mobilization_delta: 0.8,
        stamina_delta: -2.0,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::Livelihood),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::NetworkForum,
        label: "辦公開座談，讓不同團體共同列出優先事項",
        description: "資訊較透明且能建立共同議程；需要公開回應不同立場。",
        budget_cost: 2,
        support_delta: 0.07,
        trust_delta: 1.4,
        mobilization_delta: 0.4,
        stamina_delta: -2.2,
        issue: Issue::Civic,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::NetworkCoordination,
        label: "先與地方聯絡人協調行程與志工分工，並公開會後摘要",
        description: "較快建立動員網絡；若缺少透明紀錄，可能折損團隊信任。",
        budget_cost: 1,
        support_delta: 0.05,
        trust_delta: 0.1,
        mobilization_delta: 1.5,
        stamina_delta: -1.2,
        issue: Issue::Civic,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: None,
    },
];

const EVENT_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::EventVerify,
        label: "查證事件資訊，先說明已確認內容與處理期限",
        description: "降低錯誤訊息風險；較克制，但能維持公開說明的一致性。",
        budget_cost: 1,
        support_delta: 0.04,
        trust_delta: 1.8,
        mobilization_delta: 0.1,
        stamina_delta: -1.0,
        issue: Issue::Governance,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::EventService,
        label: "成立跨組工作小組，先回應受影響居民的實際需求",
        description: "要投入較多經費與人力，但可把議題處理連結到具體服務。",
        budget_cost: 3,
        support_delta: 0.08,
        trust_delta: 1.0,
        mobilization_delta: 0.7,
        stamina_delta: -2.4,
        issue: Issue::PublicHealth,
        affinity: Some(CampaignFocus::Livelihood),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::EventCounter,
        label: "以可查證資料反駁對手說法，再回到己方政策主張",
        description: "攻防反應較快、能凝聚支持者；若只剩對抗語氣，信任可能受損。",
        budget_cost: 1,
        support_delta: 0.07,
        trust_delta: -0.7,
        mobilization_delta: 0.8,
        stamina_delta: -1.0,
        issue: Issue::Media,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: None,
    },
];

const POLL_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::PollReframe,
        label: "依支持度變化調整訊息，集中說清一項可執行政策",
        description: "較靈活地回應盤面；若頻繁改主軸，既有支持者可能感到不一致。",
        budget_cost: 1,
        support_delta: 0.08,
        trust_delta: -0.2,
        mobilization_delta: 0.2,
        stamina_delta: -1.0,
        issue: Issue::Economy,
        affinity: Some(CampaignFocus::Livelihood),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::PollGroundwork,
        label: "不追逐單一數字，安排地方走訪與支持者聯繫",
        description: "增加組織接觸和投票動能；需要體力，也不會立即擴大所有群體支持。",
        budget_cost: 2,
        support_delta: 0.05,
        trust_delta: 0.5,
        mobilization_delta: 1.3,
        stamina_delta: -2.0,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::PollHoldLine,
        label: "維持原主軸，發布政見進度與成本說明",
        description: "不因單週波動大幅轉向，換取承諾一致性與可查核性。",
        budget_cost: 1,
        support_delta: 0.04,
        trust_delta: 1.2,
        mobilization_delta: 0.1,
        stamina_delta: -0.8,
        issue: Issue::Governance,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: None,
    },
];

const CLOSING_CHOICES: &[StafferChoiceSpec] = &[
    StafferChoiceSpec {
        id: StafferChoiceId::ClosingDebate,
        label: "最後一週以辯論、政見細節與資料查核收尾",
        description: "集中展示政策準備度；需要候選人與幕僚共同演練。",
        budget_cost: 2,
        support_delta: 0.08,
        trust_delta: 1.0,
        mobilization_delta: 0.2,
        stamina_delta: -2.0,
        issue: Issue::Governance,
        affinity: Some(CampaignFocus::OpenGovernment),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::ClosingField,
        label: "把剩餘資源投入掃街、志工排班與投票提醒",
        description: "強化地面接觸及動員；候選人行程與團隊體力會承受較大負擔。",
        budget_cost: 2,
        support_delta: 0.06,
        trust_delta: 0.2,
        mobilization_delta: 1.5,
        stamina_delta: -2.8,
        issue: Issue::Local,
        affinity: Some(CampaignFocus::Livelihood),
        focus: None,
    },
    StafferChoiceSpec {
        id: StafferChoiceId::ClosingCoalition,
        label: "進行最後協調，公開共同政見與合作界線",
        description: "爭取議題合作及動員整合；需要保留協商空間並清楚交代差異。",
        budget_cost: 2,
        support_delta: 0.06,
        trust_delta: 1.2,
        mobilization_delta: 0.8,
        stamina_delta: -1.8,
        issue: Issue::Civic,
        affinity: Some(CampaignFocus::DemocraticResilience),
        focus: None,
    },
];

pub fn brief_for_round(round: u8, progress: StafferProgress) -> Option<StafferBrief> {
    if round >= 12 || round & 1 == 1 {
        return None;
    }
    let index = usize::from(round / 2);
    if progress.decision(index).is_some() {
        return None;
    }
    let (title, character, scene, choices) = match index {
        0 => (
            "定調：這場選戰要讓選民記住什麼？",
            "林予安（虛構競選總幹事）",
            "候選人把第一週的行程交給團隊：有限的時間與經費，必須先決定一條能持續說明的競選主軸。",
            FOCUS_CHOICES,
        ),
        1 => (
            "陳情：先解決問題，還是先釐清責任？",
            "陳秀惠（虛構里民代表）",
            match progress.focus() {
                Some(CampaignFocus::Livelihood) => "居民帶來交通與生活成本陳情，提醒團隊主軸要回到日常需求，並提出可追蹤的處理方式。",
                Some(CampaignFocus::OpenGovernment) => "居民帶來一份權責不清的陳情，追問團隊能否先查明資料、再說明處理時程。",
                Some(CampaignFocus::DemocraticResilience) => "居民帶來跨社區的公共服務陳情，希望團隊讓受影響的人都能參與後續討論。",
                None => "居民帶來一份地方陳情，要求團隊說明誰負責、何時回覆，以及如何持續追蹤。",
            },
            PETITION_CHOICES,
        ),
        2 => (
            "地方拜會：建立信任或快速整合？",
            "吳明哲（虛構地方組織聯絡人）",
            match progress.decision(1) {
                Some(StafferDecision::Chosen(StafferChoiceId::PetitionFieldVisit)) => "前次現勘收到更多居民回報。地方團體願意協助，但希望知道團隊會如何回訪與回報進度。",
                Some(StafferDecision::Chosen(StafferChoiceId::PetitionVerify)) => "團隊已整理待查資料。地方團體願意提供線索，也希望座談不要只剩下單向說明。",
                Some(StafferDecision::Chosen(StafferChoiceId::PetitionLocalLiaison)) => "地方窗口已建立，接下來要決定採取公開座談、持續聆聽，或優先分配動員工作。",
                Some(StafferDecision::Skipped) | None => "地方組織與社區團體提出拜會邀請，團隊需要決定如何聆聽需求與安排合作。",
                _ => "地方組織與社區團體提出拜會邀請，團隊需要決定如何聆聽需求與安排合作。",
            },
            NETWORK_CHOICES,
        ),
        3 => (
            "事件應變：先回應民眾還是先處理攻防？",
            "周以晴（虛構地方記者）",
            match progress.focus() {
                Some(CampaignFocus::Livelihood) => "一項公共議題升高關注。對手幕僚方致遠提出一份成本較低的替代方案；記者追問你方除了談立場，能否提出可執行的服務安排。",
                Some(CampaignFocus::OpenGovernment) => "一項公共議題升高關注。對手幕僚方致遠公開要求你方交代資料來源；記者追問已確認資訊、待查事項與承諾期限。",
                Some(CampaignFocus::DemocraticResilience) => "一項公共議題升高關注。對手幕僚方致遠質疑你方的協作成本；記者詢問受影響社群如何參與決策。",
                None => "一項公共議題升高關注。記者要求團隊提出回應，對手也開始質疑你方處理方式。",
            },
            EVENT_CHOICES,
        ),
        4 => (
            "民調會議：看變化，還是追逐數字？",
            "林予安（虛構競選總幹事）",
            "團隊收到一份遊戲內候選人支持度摘要。單週變化可能受情境與模型波動影響；你要決定調整訊息、加強接觸，或維持主軸。",
            POLL_CHOICES,
        ),
        5 => (
            "最後布局：把資源放在辯論、地面或協調？",
            "競選團隊（虛構合成角色）",
            match progress.decision(4) {
                Some(StafferDecision::Chosen(StafferChoiceId::PollReframe)) => "支持度會議決定聚焦一項政策，但訊息調整後，團隊還得向原先支持者解釋前後差異。",
                Some(StafferDecision::Chosen(StafferChoiceId::PollGroundwork)) => "支持度會議把資源投向地方接觸；志工排班增加，候選人行程與團隊體力也更吃緊。",
                Some(StafferDecision::Chosen(StafferChoiceId::PollHoldLine)) => "支持度會議選擇維持主軸並公布政見進度；對手已準備追問未完成項目的期限。",
                Some(StafferDecision::Skipped) | None => match progress.focus() {
                    Some(CampaignFocus::Livelihood) => "剩下一週。團隊主軸是生活議題；最後要把承諾講清楚，並決定是否把人力轉向投票動員。",
                    Some(CampaignFocus::OpenGovernment) => "剩下一週。團隊主軸是透明治理；最後要展示政見細節，也要保留時間回應外界質疑。",
                    Some(CampaignFocus::DemocraticResilience) => "剩下一週。團隊主軸是民主韌性；最後要鞏固公民參與，並評估跨陣營議題協調。",
                    None => "剩下一週。候選人要你提出最後布局，在政策說明、地面動員與合作協調間做取捨。",
                },
                _ => "支持度摘要已送回團隊，最後一週要把先前的訊息與行程選擇收束成一套可執行布局。",
            },
            CLOSING_CHOICES,
        ),
        _ => return None,
    };
    Some(StafferBrief {
        index,
        week: round + 1,
        title,
        character,
        scene,
        choices,
    })
}

pub fn choice_for(index: usize, id: StafferChoiceId) -> Option<&'static StafferChoiceSpec> {
    let choices = match index {
        0 => FOCUS_CHOICES,
        1 => PETITION_CHOICES,
        2 => NETWORK_CHOICES,
        3 => EVENT_CHOICES,
        4 => POLL_CHOICES,
        5 => CLOSING_CHOICES,
        _ => return None,
    };
    choices.iter().find(|choice| choice.id == id)
}

pub fn continuity_bonus(
    index: usize,
    choice: &StafferChoiceSpec,
    progress: StafferProgress,
) -> StafferBonus {
    if index == 0 || choice.affinity != progress.focus() || progress.focus().is_none() {
        return StafferBonus::default();
    }
    if index == 5 && progress.aligned_choices_before(index) >= 3 {
        return StafferBonus {
            support_delta: 0.08,
            trust_delta: 0.5,
            note: Some("前面數次決策與競選主軸一致，團隊獲得小幅連貫加成。"),
        };
    }
    StafferBonus {
        support_delta: 0.04,
        trust_delta: 0.3,
        note: Some("這項安排延續既定競選主軸，獲得小幅連貫加成。"),
    }
}

const WEEK_ARCS: [&str; 12] = [
    "定調與組隊",
    "第一張陳情單",
    "地方網絡",
    "首波反擊",
    "數字背後",
    "辯論倒數",
    "突發事件",
    "信任測試",
    "盤面重排",
    "團隊裂縫",
    "最後協調",
    "投票日前夕",
];

// Seven compact scene beats per campaign week. The daily decision remains
// the player's action; these lines provide a changing stake and callback.
const DAILY_SCENES: [[&str; 7]; 12] = [
    [
        "開門第一天，候選人要一句能記住的主張；政策組和組織組卻帶來兩份不同簡報。",
        "第一輪行程曝光後，媒體追問承諾的經費來源，團隊發現口號比預算表走得更快。",
        "地方志工回報，居民最在意的不是標語，而是公車班次和下班後還開著的服務窗口。",
        "捐款人希望把版面留給熟悉議題，年輕志工則要求團隊回答新問題；兩邊都在等你表態。",
        "候選人行程已排滿，幕僚仍想加一場活動；體力表上，第一個警訊亮了起來。",
        "對手公布一份更短、更好懂的政見摘要，團隊得決定要逐項回應，還是把自己的主張說清楚。",
        "週末收隊時，總部牆上貼滿待辦；真正的問題是，下週要先完成哪一件。",
    ],
    [
        "一張居民陳情單送到總部，簽名很多，內容卻混著親眼所見和轉述消息。",
        "窗口找到第一位當事人，但另一位居民說法不同；團隊要先確認事實，還是先給答案？",
        "地方記者要求下午前回覆，資料部門說還差一份紀錄，候選人的行程已經在路上。",
        "對手把陳情剪成一段短片，留言開始追問責任；團隊的查證還沒完成。",
        "里民代表願意帶路現勘，但希望候選人承諾立即改善；權責單位尚未確認。",
        "回訪清單上有三個未接電話；一位志工說，答應過的回覆不能只留在會議紀錄裡。",
        "週末記者會前，團隊手上已有部分答案，也有仍待查證的空白。",
    ],
    [
        "地方組織提出拜會邀請，行程很短，條件卻是要先談清楚彼此能承諾什麼。",
        "社區團體帶來不同優先順序；每個人都支持改善，卻沒人同意先做哪一項。",
        "志工排班表出現缺口，地方聯絡人願意補人手，但希望團隊公開說明分工。",
        "閉門協調能省時間，公開座談則會多聽到不舒服的問題；兩種成本都是真的。",
        "一位長期協助地方的夥伴提醒，今天答應的事，下週居民就會拿著紀錄來追問。",
        "候選人臨時多出一個行程，交通時間將壓縮拜會；幕僚得替團隊守住優先次序。",
        "會議結束時，夥伴沒有要求保證支持，只問下一次回訪日期是否已經排好。",
    ],
    [
        "對手陣營政策幕僚方致遠（虛構）把焦點轉向你方尚未說明的細節，這次質疑有一部分確實成立。",
        "剪輯過的發言在社群流傳；原始影片能澄清語境，但不會讓失言自動消失。",
        "發言人想立刻回擊，政策幕僚要求先核對數字；候選人只剩幾分鐘決定先說什麼。",
        "一份舊資料被重新翻出，團隊得辨認它是新證據，還是舊爭議換了包裝。",
        "查核結果支持你方部分說法，也推翻另一段說法；完整答案比單句標題更難傳開。",
        "記者會問題比預演尖銳，候選人沒有照稿回答；幕僚要不要補充，還是先讓回答留下？",
        "週末新聞已經定調，團隊可以爭取第二次說明，但不能假裝第一次沒有發生。",
    ],
    [
        "支持度摘要出爐：總體數字變動不大，分眾原型卻朝不同方向移動。",
        "一位幕僚主張追逐最新波動，另一位提醒樣本和情境都有限；會議室安靜下來。",
        "地方行程反映出一個沒有出現在總表上的問題，團隊要不要重排本週主題？",
        "候選人看到最弱的一項指標，想加開活動；財務人員把剩餘預算推到桌子中央。",
        "一場訪談得到大量點閱，卻沒有回答原本的政策問題；聲量與說服不是同一件事。",
        "新的整理方式讓差異比較清楚，但每個百分點仍是遊戲模型，不是現實民調結論。",
        "週報最後一欄寫著『下週再追』；團隊必須把它改成一個今天就能執行的決定。",
    ],
    [
        "辯論題目公布，候選人想逐題準備，幕僚只剩時間把三個核心答案磨到清楚。",
        "模擬提問碰到政見成本，政策組答得精確，卻超過了鏡頭前能聽完的時間。",
        "候選人練習反問時，教練提醒：指出差異有用，把對手說成稻草人只會讓答案失焦。",
        "方致遠的主張並非全無道理；要贏得比較，得先把對方最強的一點講準。",
        "服裝、交通和發言順序都已排妥，唯獨一個追問尚未找到不閃躲的回答。",
        "辯論結束後，團隊收到掌聲和批評；真正的考驗是明天能不能把承諾接回行程。",
        "剪輯片段開始流傳，政策全文也同步上線；團隊要讓兩者都找得到。",
    ],
    [
        "早晨的事件快訊打亂原定行程，受到影響的居民先需要協助，不是競選標語。",
        "資訊來源彼此矛盾，發言群組已經有人想下結論；查證窗口要求再等一小時。",
        "候選人要不要取消公開活動？現場需要人手，團隊也要避免把關懷變成拍照背景。",
        "專業人員給了暫時說法，仍保留幾項未知；記者正等團隊把不確定之處說出來。",
        "對手提出不同處理方案，其中一項成本比你方低；政策組要求正面比較，不要只攻擊動機。",
        "居民回報服務窗口已接通，另一處仍沒有進展；團隊不能把一個好消息說成全部解決。",
        "深夜整理資料時，最受轉傳的一句話不是最重要的一句；團隊重新排了明天的說明順序。",
    ],
    [
        "專訪主持人把同一題問了兩次，答案都正確，第二次卻更像在迴避。",
        "一則未查證的截圖被轉傳；查核人員找到原文，但修正訊息傳得比截圖慢。",
        "志工反映，選民想知道政見怎麼付錢；發言稿卻還在講願景。",
        "政策團隊把成本表攤開，候選人擔心數字太複雜；幕僚得在完整和易懂間取捨。",
        "地方團體願意共同說明，但不想被剪成替任何一方背書的畫面。",
        "一場公開活動的提問超出預排範圍，候選人選擇留下來聽完。",
        "週末回顧沒有漂亮結論，只有三項可以核對的承諾和一項仍待查證的問題。",
    ],
    [
        "新一輪支持度摘要顯示差距縮小；團隊先核對起始盤與本週變化，沒有立刻宣布趨勢。",
        "一個原型群體的支持上升，另一群體回落；行程表沒有足夠空格同時照顧全部議題。",
        "媒體想要一個『誰領先』的標題，幕僚更想知道哪項承諾尚未被理解。",
        "候選人問是否要換主軸，總幹事回答：先看這是短期波動，還是連續幾週都出現的訊號。",
        "地方回訪補上了數字看不見的原因：問題不是沒聽過，而是不相信做得到。",
        "財務表和支持度表放在一起後，團隊發現最有效的方案也是最耗資源的方案。",
        "週會最後只留下一個調整：訊息要更具體，但不能讓上一週的承諾消失。",
    ],
    [
        "組織組想加一場造勢，政策組堅持先公布成本；兩邊爭的都是同一個有限時段。",
        "一位夥伴要求公開合作界線，另一位擔心談判空間被鎖死；協調沒有立即答案。",
        "候選人疲勞開始影響行程品質，幕僚得刪掉一場活動，並向邀請方說明原因。",
        "內部會議紀錄外流，內容沒有違規，卻讓合作方擔心承諾是否會被斷章取義。",
        "方致遠提出一項更受歡迎的方案；你的團隊可以批評，也可以承認它值得比較。",
        "志工問誰能拍板，會議才發現兩個小組都以為對方負責。",
        "夜間檢討沒有指定戰犯，改把責任、期限和回報窗口寫進同一張表。",
    ],
    [
        "合作夥伴帶著最後一份修正案到總部，接受它會改變政見排序，不接受則可能各自出發。",
        "候選人行程只剩三個空檔；辯論準備、地方回訪和志工訓練都在爭這些時間。",
        "團隊再次檢查競選主軸，確認最後一週不會突然承諾一項沒有成本說明的政策。",
        "地方聯絡人願意擴大動員，但要求公開共同政見與彼此不同的地方。",
        "一名志工把排班表貼上牆，空白格比預期多；距投票日只剩幾天。",
        "候選人問最後要去哪裡，幕僚把交通時間、支持度差異和體力一項項列出。",
        "最後一次策略會議結束，沒有哪條路能同時保住所有資源；團隊必須選擇承擔哪種代價。",
    ],
    [
        "投票日前一週，團隊把每個承諾再核對一次；最危險的不是忘詞，而是說出無法兌現的話。",
        "志工問候選人今天還能走幾個路口，行程表第一次把休息列成正式工作。",
        "最後一場訪談追問勝負，候選人把答案帶回政見；幕僚確認現場沒有多加一句未審核承諾。",
        "對手陣營也在動員，雙方都把焦點放在投票率；選舉不是只有看支持度。",
        "居民回傳一張已處理問題的照片，也附上一項仍未完成的要求。",
        "投票提醒、交通安排、志工輪班逐一確認；所有人都知道這些努力不保證結果。",
        "競選團隊關上總部的燈前，桌上留下明天的聯絡清單和一份尚未拆封的回報。",
    ],
];

pub fn daily_scene(week: u8, day: u8) -> Option<(&'static str, &'static str)> {
    if !(1..=12).contains(&week) || !(1..=7).contains(&day) {
        return None;
    }
    Some((
        WEEK_ARCS[usize::from(week - 1)],
        DAILY_SCENES[usize::from(week - 1)][usize::from(day - 1)],
    ))
}

/// A once-per-arc callback that lets weekly staff decisions leave a visible
/// mark on the daily campaign story without adding another resource action.
pub fn daily_route_echo(week: u8, day: u8, progress: StafferProgress) -> Option<&'static str> {
    let decision_index = match (week, day) {
        (1, 2) => 0,
        (3, 1) => 1,
        (5, 1) => 2,
        (7, 1) => 3,
        (9, 1) => 4,
        (11, 1) => 5,
        _ => return None,
    };
    let decision = progress.decision(decision_index)?;
    Some(match decision {
        StafferDecision::Skipped => match decision_index {
            0 => "林予安沒有替團隊定下主軸，白板上仍留著三種說法；今天每個人都得說明自己為何選擇其中一種。",
            1 => "陳秀惠把尚未回覆的陳情副本留在總部。沒有幕僚會議結論，地方窗口仍照原本流程追蹤。",
            2 => "吳明哲照常寄來拜會邀請，卻沒有收到明確分工；地方夥伴各自排定行程。",
            3 => "周以晴把待查事項列進報導，團隊沒有統一回應；每位發言人都要自行核對資料。",
            4 => "林予安把支持度摘要收進資料夾，沒有追逐單週變化；團隊沿用既有安排，等待下一次檢視。",
            _ => "最後布局沒有新的協調決定。志工仍照既有排班出發，總部把尚未處理的承諾交給投票日後的團隊。",
        },
        StafferDecision::Chosen(id) => match id {
            StafferChoiceId::FocusLivelihood => "陳秀惠在白板旁補上一句：「回覆居民的日期也要寫上去。」從今天起，民生主張旁多了一欄追蹤期限。",
            StafferChoiceId::FocusOpenGovernment => "周以晴把查核表收進採訪資料：「沒完成的部分也請列出來。」團隊開始把進度和空白一起公開。",
            StafferChoiceId::FocusDemocracy => "志工把座談名單貼上牆，林予安提醒：「被影響的人要坐在桌邊，不只出現在海報上。」",
            StafferChoiceId::PetitionFieldVisit => "現勘照片送回總部，陳秀惠圈出尚未改善的路段：「我會帶居民回來看你們有沒有再來。」",
            StafferChoiceId::PetitionVerify => "查核表把已確認和待確認分成兩欄。周以晴說：「我會追問，但不把還不知道的寫成答案。」",
            StafferChoiceId::PetitionLocalLiaison => "地方窗口終於有了姓名和回覆期限；吳明哲提醒，建立聯絡網只是開始，回電才算數。",
            StafferChoiceId::NetworkListening => "吳明哲帶回一疊不同版本的需求清單。林予安沒有合併掉分歧，改把下次回訪日期逐一寫上。",
            StafferChoiceId::NetworkForum => "座談紀錄公開後，兩個團體對優先順序仍有不同意見；團隊承諾把差異留在紀錄，不剪成一致口號。",
            StafferChoiceId::NetworkCoordination => "分工表讓志工很快找到位置，吳明哲也要求公布責任窗口；沒有人能再把漏接的回覆推給別組。",
            StafferChoiceId::EventVerify => "周以晴拿到逐項核對的回應稿：「這次我會把已知、未知和期限一起播。」團隊先把速度讓給可信度。",
            StafferChoiceId::EventService => "服務小組先接起居民電話，候選人的公開行程往後挪了半小時；林予安要求不要把現場協助變成拍照背景。",
            StafferChoiceId::EventCounter => "反駁資料如期發布，但林予安刪掉一句揣測動機的話：「我們要攻的是說法，不是替對方編故事。」",
            StafferChoiceId::PollReframe => "林予安把訊息改成一項可執行承諾，並請發言人逐一向原支持者解釋為何調整，而不是假裝從未改變。",
            StafferChoiceId::PollGroundwork => "志工排班表多了幾個名字，也多了幾個空白；地方回訪開始補上支持度摘要看不見的原因。",
            StafferChoiceId::PollHoldLine => "政見進度和成本表一併上線。周以晴提醒候選人，公開承諾之後，未完成的部分也會被逐項追問。",
            StafferChoiceId::ClosingDebate => "最後一次演練沒有刪去難題。候選人對著空椅子重答一次，林予安只說：「別把還沒做到的說成已經完成。」",
            StafferChoiceId::ClosingField => "志工接過最後一輪投票提醒名單，陳秀惠留下尚待處理的陳情：「選舉結束後，這張單也要有人接。」",
            StafferChoiceId::ClosingCoalition => "合作夥伴在共同政見旁簽名，也保留各自的差異；吳明哲把公開版本送到地方群組，沒有替任何人背書。",
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staffer_briefings_are_six_biweekly_decisions_with_three_choices() {
        let progress = StafferProgress::default();
        for round in 0..12 {
            let brief = brief_for_round(round, progress);
            if round % 2 == 0 {
                let brief = brief.expect("every other week has a staff briefing");
                assert_eq!(brief.week, round + 1);
                assert_eq!(brief.choices.len(), 3);
            } else {
                assert!(brief.is_none());
            }
        }
    }

    #[test]
    fn campaign_focus_can_reward_consistent_later_choices() {
        let mut progress = StafferProgress::default();
        let theme = choice_for(0, StafferChoiceId::FocusOpenGovernment).unwrap();
        assert!(progress.record(0, StafferDecision::Chosen(theme.id), theme.focus));
        let verify = choice_for(1, StafferChoiceId::PetitionVerify).unwrap();
        assert_eq!(continuity_bonus(1, verify, progress).support_delta, 0.04);
        assert!(progress.record(1, StafferDecision::Chosen(verify.id), verify.focus));
        let forum = choice_for(2, StafferChoiceId::NetworkForum).unwrap();
        assert!(progress.record(2, StafferDecision::Chosen(forum.id), forum.focus));
        let hold_line = choice_for(4, StafferChoiceId::PollHoldLine).unwrap();
        assert!(progress.record(4, StafferDecision::Chosen(hold_line.id), hold_line.focus));
        let closing = choice_for(5, StafferChoiceId::ClosingDebate).unwrap();
        let bonus = continuity_bonus(5, closing, progress);
        assert_eq!(bonus.support_delta, 0.08);
        assert!(bonus.note.is_some());
    }

    #[test]
    fn staffer_choices_change_later_briefings_and_the_closing_scene() {
        let mut progress = StafferProgress::default();
        let focus = choice_for(0, StafferChoiceId::FocusLivelihood).unwrap();
        progress.record(0, StafferDecision::Chosen(focus.id), focus.focus);
        assert!(brief_for_round(2, progress)
            .unwrap()
            .scene
            .contains("生活成本"));

        let petition = choice_for(1, StafferChoiceId::PetitionFieldVisit).unwrap();
        progress.record(1, StafferDecision::Chosen(petition.id), petition.focus);
        assert!(brief_for_round(4, progress).unwrap().scene.contains("現勘"));

        let network = choice_for(2, StafferChoiceId::NetworkListening).unwrap();
        progress.record(2, StafferDecision::Chosen(network.id), network.focus);
        let event = choice_for(3, StafferChoiceId::EventService).unwrap();
        progress.record(3, StafferDecision::Chosen(event.id), event.focus);
        let poll = choice_for(4, StafferChoiceId::PollGroundwork).unwrap();
        progress.record(4, StafferDecision::Chosen(poll.id), poll.focus);
        assert!(brief_for_round(10, progress)
            .unwrap()
            .scene
            .contains("志工排班"));
        let closing = choice_for(5, StafferChoiceId::ClosingField).unwrap();
        progress.record(5, StafferDecision::Chosen(closing.id), closing.focus);
        assert!(progress.ending_note().contains("居民回覆清單"));
    }

    #[test]
    fn staffer_route_is_recordable_and_includes_skips() {
        let mut progress = StafferProgress::default();
        let theme = choice_for(0, StafferChoiceId::FocusLivelihood).unwrap();
        assert!(progress.record(0, StafferDecision::Chosen(theme.id), theme.focus));
        assert!(progress.record(1, StafferDecision::Skipped, None));
        assert_eq!(progress.completed_briefings(), 2);
        assert!(progress.summary().contains("skip"));
        assert!(progress.is_valid(2));
        assert!(!progress.is_valid(0));
        assert!(progress.ending_note().contains("投票日前夕"));
    }

    #[test]
    fn daily_campaign_scenes_cover_all_84_days_and_escalate_by_week() {
        for week in 1..=12 {
            for day in 1..=7 {
                let (arc, scene) = daily_scene(week, day).unwrap();
                assert!(!arc.is_empty());
                assert!(scene.chars().count() > 20);
            }
        }
        assert!(daily_scene(0, 1).is_none());
        assert!(daily_scene(13, 1).is_none());
        assert!(daily_scene(1, 8).is_none());
    }

    #[test]
    fn daily_route_echoes_reflect_choices_and_only_appear_at_story_beats() {
        let mut progress = StafferProgress::default();
        assert!(daily_route_echo(1, 2, progress).is_none());

        let focus = choice_for(0, StafferChoiceId::FocusLivelihood).unwrap();
        progress.record(0, StafferDecision::Chosen(focus.id), focus.focus);
        assert!(daily_route_echo(1, 2, progress)
            .unwrap()
            .contains("回覆居民的日期"));
        assert!(daily_route_echo(1, 3, progress).is_none());

        let verify = choice_for(1, StafferChoiceId::PetitionVerify).unwrap();
        progress.record(1, StafferDecision::Chosen(verify.id), verify.focus);
        assert!(daily_route_echo(3, 1, progress).unwrap().contains("待確認"));

        progress.record(2, StafferDecision::Skipped, None);
        assert!(daily_route_echo(5, 1, progress)
            .unwrap()
            .contains("拜會邀請"));
    }
}
