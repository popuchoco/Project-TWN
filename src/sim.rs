use crate::history::{
    baseline_election, challenger_candidate_index, events_for_campaign, incumbent_candidate_index,
    EventCard, Issue, PresidentialElection, RegionProfile,
};

pub const CAMPAIGN_ROUNDS: u8 = 12;
const DEFAULT_SEED: u64 = 0x0054_574E_5F53_494D;
const MAX_BUDGET: i32 = 120;
const WEEKLY_FUNDING: i32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Incumbent,
    Challenger,
}

impl Side {
    pub fn label(self) -> &'static str {
        match self {
            Side::Incumbent => "尋求連任／執政方",
            Side::Challenger => "在野挑戰方",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignStyle {
    Balanced,
    Policy,
    Organizer,
    Communicator,
}

impl CampaignStyle {
    pub fn label(self) -> &'static str {
        match self {
            CampaignStyle::Balanced => "均衡型",
            CampaignStyle::Policy => "政策型",
            CampaignStyle::Organizer => "組織型",
            CampaignStyle::Communicator => "溝通型",
        }
    }

    fn attributes(self) -> CandidateAttributes {
        match self {
            CampaignStyle::Balanced => CandidateAttributes {
                policy: 50.0,
                organization: 50.0,
                communication: 50.0,
            },
            CampaignStyle::Policy => CandidateAttributes {
                policy: 62.0,
                organization: 48.0,
                communication: 50.0,
            },
            CampaignStyle::Organizer => CandidateAttributes {
                policy: 48.0,
                organization: 62.0,
                communication: 50.0,
            },
            CampaignStyle::Communicator => CandidateAttributes {
                policy: 50.0,
                organization: 48.0,
                communication: 62.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CandidateAttributes {
    pub policy: f64,
    pub organization: f64,
    pub communication: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionId {
    PolicyDelivery,
    CrisisResponse,
    TransparencyReport,
    PublicServices,
    CommunityTour,
    PolicyForum,
    LegalAssembly,
    CoalitionTalks,
    OversightPetition,
    PublicMessage,
    RestAndPrepare,
    PollCoordination,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionSpec {
    pub id: ActionId,
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub base_swing: f64,
    pub trust_delta: f64,
    pub turnout_delta: f64,
    pub issues: &'static [Issue],
    pub stamina_cost: f64,
    pub stamina_recovery: f64,
}

const INCUMBENT_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: ActionId::PolicyDelivery,
        name: "施政方案落地",
        description: "以可量化的政策進度與公共服務表現爭取支持。",
        cost: 20,
        base_swing: 1.75,
        trust_delta: 0.8,
        turnout_delta: 0.4,
        issues: &[Issue::Economy, Issue::Governance, Issue::Local],
        stamina_cost: 10.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CrisisResponse,
        name: "公共危機應變",
        description: "提出防災、衛生或民生危機應變計畫；成效與透明度影響信任。",
        cost: 18,
        base_swing: 1.75,
        trust_delta: 1.8,
        turnout_delta: 0.5,
        issues: &[Issue::PublicHealth, Issue::Governance],
        stamina_cost: 12.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::TransparencyReport,
        name: "公開施政與檢討",
        description: "公開進度、預算及尚未完成事項，降低資訊落差。",
        cost: 10,
        base_swing: 1.2,
        trust_delta: 2.4,
        turnout_delta: 0.4,
        issues: &[Issue::Trust, Issue::Media, Issue::Governance],
        stamina_cost: 5.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicServices,
        name: "地方民生服務",
        description: "強化公共運輸、照護、教育或社區服務交付。",
        cost: 14,
        base_swing: 1.45,
        trust_delta: 0.9,
        turnout_delta: 1.0,
        issues: &[Issue::Local, Issue::Education, Issue::Rights],
        stamina_cost: 9.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicMessage,
        name: "公開政策問答",
        description: "以低成本發布可核實的政策說明與問答。",
        cost: 0,
        base_swing: 0.45,
        trust_delta: 0.2,
        turnout_delta: 0.25,
        issues: &[Issue::Trust, Issue::Media],
        stamina_cost: 2.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::RestAndPrepare,
        name: "休息與政見準備",
        description: "恢復體力，整理政策資料並為下一週做準備。",
        cost: 0,
        base_swing: 0.2,
        trust_delta: 0.1,
        turnout_delta: 0.0,
        issues: &[],
        stamina_cost: 0.0,
        stamina_recovery: 22.0,
    },
    ActionSpec {
        id: ActionId::PollCoordination,
        name: "對比／互比民調與整合協商",
        description: "檢視多候選人民調與對手互比，再嘗試整合一位落後候選人的支持。",
        cost: 12,
        base_swing: 0.7,
        trust_delta: 0.6,
        turnout_delta: 0.6,
        issues: &[Issue::Governance, Issue::Trust],
        stamina_cost: 7.0,
        stamina_recovery: 0.0,
    },
];

const CHALLENGER_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        id: ActionId::CommunityTour,
        name: "社區走訪與傾聽",
        description: "進行公開的地方走訪，把居民回饋納入政策提案。",
        cost: 12,
        base_swing: 1.45,
        trust_delta: 1.0,
        turnout_delta: 1.2,
        issues: &[Issue::Local, Issue::Rights, Issue::Education],
        stamina_cost: 10.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PolicyForum,
        name: "政見論壇與辯論",
        description: "以可檢驗的政策選項與對手比較，強化政策可信度。",
        cost: 12,
        base_swing: 1.65,
        trust_delta: 1.4,
        turnout_delta: 0.6,
        issues: &[
            Issue::Economy,
            Issue::CrossStrait,
            Issue::Governance,
            Issue::PublicHealth,
        ],
        stamina_cost: 8.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::LegalAssembly,
        name: "合法集會／公民行動",
        description: "舉辦和平、依法申請的集會，提升議題能見度與參與。",
        cost: 8,
        base_swing: 1.35,
        trust_delta: 0.4,
        turnout_delta: 2.0,
        issues: &[Issue::Civic, Issue::Media, Issue::Education, Issue::Rights],
        stamina_cost: 13.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::CoalitionTalks,
        name: "跨黨協調與組織整合",
        description: "協調理念相近的團體與地方組織，擴大合作基礎。",
        cost: 14,
        base_swing: 1.25,
        trust_delta: 1.0,
        turnout_delta: 0.8,
        issues: &[Issue::Civic, Issue::Governance, Issue::CrossStrait],
        stamina_cost: 7.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::OversightPetition,
        name: "監督提案與公開連署",
        description: "透過可查證資料提出監督議題、具體替代方案與公開連署。",
        cost: 9,
        base_swing: 1.45,
        trust_delta: 1.0,
        turnout_delta: 0.6,
        issues: &[Issue::Trust, Issue::Media, Issue::Governance],
        stamina_cost: 7.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::PublicMessage,
        name: "公開政策問答",
        description: "以低成本發布可核實的政策說明與問答。",
        cost: 0,
        base_swing: 0.45,
        trust_delta: 0.2,
        turnout_delta: 0.25,
        issues: &[Issue::Trust, Issue::Media],
        stamina_cost: 2.0,
        stamina_recovery: 0.0,
    },
    ActionSpec {
        id: ActionId::RestAndPrepare,
        name: "休息與政見準備",
        description: "恢復體力，整理政策資料並為下一週做準備。",
        cost: 0,
        base_swing: 0.2,
        trust_delta: 0.1,
        turnout_delta: 0.0,
        issues: &[],
        stamina_cost: 0.0,
        stamina_recovery: 22.0,
    },
    ActionSpec {
        id: ActionId::PollCoordination,
        name: "對比／互比民調與整合協商",
        description: "檢視多候選人民調與對手互比，再嘗試整合一位落後候選人的支持。",
        cost: 12,
        base_swing: 0.7,
        trust_delta: 0.6,
        turnout_delta: 0.6,
        issues: &[Issue::Governance, Issue::Trust],
        stamina_cost: 7.0,
        stamina_recovery: 0.0,
    },
];

pub fn actions_for(side: Side) -> &'static [ActionSpec] {
    match side {
        Side::Incumbent => INCUMBENT_ACTIONS,
        Side::Challenger => CHALLENGER_ACTIONS,
    }
}

#[derive(Clone, Debug)]
pub struct SimCandidate {
    pub name: String,
    pub party: &'static str,
    pub historical_share: f64,
}

#[derive(Clone, Debug)]
pub struct GameState {
    pub campaign_year: u16,
    pub baseline_year: u16,
    pub region: Option<&'static RegionProfile>,
    pub side: Side,
    pub style: CampaignStyle,
    pub player_attributes: CandidateAttributes,
    pub player_index: usize,
    pub candidates: Vec<SimCandidate>,
    pub polls: Vec<f64>,
    pub budget: i32,
    pub trust: f64,
    pub turnout_momentum: f64,
    pub stamina: f64,
    pub round: u8,
    pub events: Vec<&'static EventCard>,
    action_uses: [u8; 12],
    matched_events: u8,
    rng: Rng,
}

#[derive(Clone, Debug)]
pub struct ActionOutcome {
    pub action_name: &'static str,
    pub event_title: Option<&'static str>,
    pub share_before: f64,
    pub share_after: f64,
    pub trust_after: f64,
    pub turnout_after: f64,
    pub stamina_after: f64,
    pub cost: i32,
    pub weekly_funding: i32,
    pub budget_after: i32,
    pub coordination_note: Option<String>,
    pub round: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct CampaignReview {
    pub policy: f64,
    pub trust: f64,
    pub mobilization: f64,
    pub event_response: f64,
    pub resource_management: f64,
    pub versatility: f64,
}

impl CampaignReview {
    pub fn axes(self) -> [(&'static str, f64); 6] {
        [
            ("政策準備", self.policy),
            ("信任經營", self.trust),
            ("組織動員", self.mobilization),
            ("議題回應", self.event_response),
            ("資源管理", self.resource_management),
            ("行動多樣", self.versatility),
        ]
    }
}

#[derive(Clone, Debug)]
pub struct ElectionOutcome {
    pub shares: Vec<f64>,
    pub winner_index: usize,
    pub player_win_probability: f64,
}

#[derive(Clone, Copy, Debug)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) / ((1_u64 << 53) as f64)
    }

    fn between(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.unit()
    }
}

impl GameState {
    pub fn new(campaign_year: u16, side: Side, seed: u64) -> Self {
        Self::new_with_style(campaign_year, side, seed, CampaignStyle::Balanced)
    }

    pub fn new_with_style(campaign_year: u16, side: Side, seed: u64, style: CampaignStyle) -> Self {
        let election = baseline_election(campaign_year);
        Self::from_election(campaign_year, side, election, seed, style)
    }

    pub fn new_local_with_style(
        campaign_year: u16,
        side: Side,
        seed: u64,
        style: CampaignStyle,
        region: &'static RegionProfile,
    ) -> Self {
        let mut game =
            Self::from_election(campaign_year, side, baseline_election(2024), seed, style);
        game.baseline_year = 2022;
        game.region = Some(region);
        let (parties, shares) = local_slate(region);
        game.candidates = parties
            .iter()
            .zip(shares)
            .map(|(party, share)| SimCandidate {
                name: format!("{party}（地方情境代表）"),
                party,
                historical_share: share,
            })
            .collect();
        game.polls = shares.to_vec();
        game.player_index = match side {
            Side::Incumbent => 0,
            Side::Challenger => 1,
        };
        game
    }

    fn from_election(
        campaign_year: u16,
        side: Side,
        election: &'static PresidentialElection,
        seed: u64,
        style: CampaignStyle,
    ) -> Self {
        let player_index = match side {
            Side::Incumbent => incumbent_candidate_index(election),
            Side::Challenger => challenger_candidate_index(election),
        };
        let uses_historical_names = campaign_year == election.year;
        let candidates: Vec<SimCandidate> = election
            .candidates
            .iter()
            .map(|result| SimCandidate {
                name: if uses_historical_names {
                    format!("{} / {}", result.president, result.vice_president)
                } else {
                    format!("{}（情境代表）", result.party)
                },
                party: result.party,
                historical_share: result.share_pct,
            })
            .collect();
        let mut polls: Vec<f64> = candidates
            .iter()
            .map(|candidate| candidate.historical_share)
            .collect();
        normalize(&mut polls);
        let (trust, turnout_momentum) = match side {
            Side::Incumbent => (54.0, 50.0),
            Side::Challenger => (50.0, 50.0),
        };

        Self {
            campaign_year,
            baseline_year: election.year,
            region: None,
            side,
            style,
            player_attributes: style.attributes(),
            player_index,
            candidates,
            polls,
            budget: 100,
            trust,
            turnout_momentum,
            stamina: 100.0,
            round: 0,
            events: events_for_campaign(campaign_year),
            action_uses: [0; 12],
            matched_events: 0,
            rng: Rng::new(seed),
        }
    }

    pub fn current_event(&self) -> Option<&'static EventCard> {
        if self.events.is_empty() {
            return None;
        }
        Some(self.events[(self.round as usize) % self.events.len()])
    }

    pub fn apply_action(&mut self, id: ActionId) -> Result<ActionOutcome, String> {
        if self.round >= CAMPAIGN_ROUNDS {
            return Err("競選期已結束。".to_string());
        }
        let action = actions_for(self.side)
            .iter()
            .find(|action| action.id == id)
            .ok_or_else(|| "這項行動不適用於目前陣營。".to_string())?;
        if self.budget < action.cost {
            return Err(format!(
                "預算不足（需要 {}，剩餘 {}）。",
                action.cost, self.budget
            ));
        }
        if id == ActionId::PollCoordination && self.candidates.len() < 3 {
            return Err("至少三位候選人時才有整合協商議題。".to_string());
        }

        let share_before = self.polls[self.player_index];
        let current_event = self.current_event();
        let event_title = current_event.map(|event| event.title);
        let event_match = current_event
            .filter(|event| action.issues.contains(&event.issue))
            .is_some();
        let issue_bonus = if event_match { 0.65 } else { 0.0 };
        let local_match = self
            .region
            .filter(|region| action.issues.contains(&region.focus))
            .map(|_| 0.4)
            .unwrap_or(0.0);
        let trust_bonus = ((self.trust - 50.0) * 0.015).clamp(-0.45, 0.55);
        let skill = match action.id {
            ActionId::PolicyDelivery | ActionId::CrisisResponse | ActionId::PolicyForum => {
                self.player_attributes.policy
            }
            ActionId::CommunityTour
            | ActionId::PublicServices
            | ActionId::LegalAssembly
            | ActionId::CoalitionTalks => self.player_attributes.organization,
            ActionId::TransparencyReport
            | ActionId::OversightPetition
            | ActionId::PublicMessage
            | ActionId::RestAndPrepare
            | ActionId::PollCoordination => self.player_attributes.communication,
        };
        let skill_bonus = ((skill - 50.0) * 0.02).clamp(-0.6, 0.6);
        let fatigue_penalty = if self.stamina < 20.0 {
            -0.55
        } else if self.stamina < 45.0 {
            -0.25
        } else {
            0.0
        };
        let jitter = self.rng.between(-0.25, 0.25);
        let swing = if id == ActionId::RestAndPrepare {
            0.0
        } else {
            (action.base_swing
                + issue_bonus
                + local_match
                + trust_bonus
                + skill_bonus
                + fatigue_penalty
                + jitter)
                .clamp(0.1, 3.2)
        };

        self.budget -= action.cost;
        shift_share(&mut self.polls, self.player_index, swing);

        let mut coordination_note = None;
        if id == ActionId::PollCoordination {
            if let Some(target) = self.weakest_rival() {
                let success_chance = (0.42
                    + (self.trust - 50.0) * 0.006
                    + (self.player_attributes.communication - 50.0) * 0.008)
                    .clamp(0.18, 0.82);
                if self.rng.unit() < success_chance {
                    let transfer = self.polls[target] * 0.16;
                    transfer_share(&mut self.polls, target, self.player_index, transfer);
                    coordination_note = Some(format!(
                        "協商成功：{} 的部分支持轉向玩家陣營（模型轉移 {:.1} 個百分點）",
                        self.candidates[target].party, transfer
                    ));
                } else {
                    coordination_note = Some(format!(
                        "互比結果已完成，但與 {} 的整合協商未達共識",
                        self.candidates[target].party
                    ));
                }
            }
        }

        // A competing campaign reacts every round. This small, seeded counter-
        // swing represents ordinary campaign competition rather than an event
        // assigning a partisan outcome.
        if id != ActionId::RestAndPrepare {
            if let Some(rival_index) = self.strongest_rival() {
                let counter_swing = self.rng.between(0.08, 0.42);
                shift_share(&mut self.polls, rival_index, counter_swing);
            }
        }

        self.trust = (self.trust + action.trust_delta).clamp(0.0, 100.0);
        self.turnout_momentum = (self.turnout_momentum + action.turnout_delta).clamp(0.0, 100.0);
        self.stamina =
            (self.stamina - action.stamina_cost + action.stamina_recovery).clamp(0.0, 100.0);
        self.action_uses[action_index(id)] = self.action_uses[action_index(id)].saturating_add(1);
        if event_match {
            self.matched_events = self.matched_events.saturating_add(1);
        }
        let before_funding = self.budget;
        self.budget = (self.budget + WEEKLY_FUNDING).min(MAX_BUDGET);
        let weekly_funding = self.budget - before_funding;
        self.round += 1;
        normalize(&mut self.polls);

        Ok(ActionOutcome {
            action_name: action.name,
            event_title,
            share_before,
            share_after: self.polls[self.player_index],
            trust_after: self.trust,
            turnout_after: self.turnout_momentum,
            stamina_after: self.stamina,
            cost: action.cost,
            weekly_funding,
            budget_after: self.budget,
            coordination_note,
            round: self.round,
        })
    }

    pub fn phase_label(&self) -> &'static str {
        match self.round / 4 {
            0 => "布局期",
            1 => "議題攻防期",
            _ => "決戰期",
        }
    }

    pub fn review(&self) -> CampaignReview {
        let policy_actions = [
            ActionId::PolicyDelivery,
            ActionId::CrisisResponse,
            ActionId::PolicyForum,
        ]
        .iter()
        .map(|id| self.action_uses[action_index(*id)] as f64)
        .sum::<f64>();
        let unique_actions = self.action_uses.iter().filter(|count| **count > 0).count() as f64;
        CampaignReview {
            policy: (35.0 + policy_actions * 8.0).clamp(0.0, 100.0),
            trust: self.trust,
            mobilization: self.turnout_momentum,
            event_response: (35.0 + self.matched_events as f64 * 12.0).clamp(0.0, 100.0),
            resource_management: (55.0 + (self.budget as f64 * 0.25)).clamp(0.0, 100.0),
            versatility: (unique_actions * (100.0 / 7.0)).clamp(0.0, 100.0),
        }
    }

    pub fn finish(&mut self) -> ElectionOutcome {
        let mut player_wins = 0_usize;
        let mut final_shares = self.polls.clone();
        let samples = 1_200;
        // Mobilization changes the player's likely vote yield slightly rather
        // than merely appearing as a decorative meter (bounded at +/- 2 pts).
        let mobilization_effect = ((self.turnout_momentum - 50.0) * 0.04).clamp(-2.0, 2.0);
        for sample in 0..samples {
            let mut ballot = self.polls.clone();
            let late_swing = self.rng.between(-4.0, 4.0) + mobilization_effect;
            shift_share(&mut ballot, self.player_index, late_swing);
            if sample == 0 {
                final_shares = ballot.clone();
            }
            if winner_index(&ballot) == self.player_index {
                player_wins += 1;
            }
        }

        ElectionOutcome {
            winner_index: winner_index(&final_shares),
            shares: final_shares,
            player_win_probability: player_wins as f64 / samples as f64,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.round >= CAMPAIGN_ROUNDS
    }

    fn strongest_rival(&self) -> Option<usize> {
        self.polls
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.player_index)
            .max_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
    }

    fn weakest_rival(&self) -> Option<usize> {
        self.polls
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.player_index)
            .min_by(|(_, left), (_, right)| left.total_cmp(right))
            .map(|(index, _)| index)
    }
}

fn winner_index(shares: &[f64]) -> usize {
    shares
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn action_index(id: ActionId) -> usize {
    match id {
        ActionId::PolicyDelivery => 0,
        ActionId::CrisisResponse => 1,
        ActionId::TransparencyReport => 2,
        ActionId::PublicServices => 3,
        ActionId::CommunityTour => 4,
        ActionId::PolicyForum => 5,
        ActionId::LegalAssembly => 6,
        ActionId::CoalitionTalks => 7,
        ActionId::OversightPetition => 8,
        ActionId::PublicMessage => 9,
        ActionId::RestAndPrepare => 10,
        ActionId::PollCoordination => 11,
    }
}

fn local_slate(region: &RegionProfile) -> ([&'static str; 3], [f64; 3]) {
    match region.previous_winner_party {
        "民進黨" => (["民進黨", "國民黨", "民眾黨"], [43.0, 37.0, 20.0]),
        "民眾黨" => (["民眾黨", "民進黨", "國民黨"], [39.0, 33.0, 28.0]),
        "無黨籍" => (["無黨籍", "國民黨", "民進黨"], [38.0, 34.0, 28.0]),
        _ => (["國民黨", "民進黨", "民眾黨"], [43.0, 37.0, 20.0]),
    }
}

fn shift_share(shares: &mut [f64], candidate_index: usize, delta: f64) {
    if shares.len() < 2 || candidate_index >= shares.len() || delta.abs() < f64::EPSILON {
        return;
    }
    let old = shares[candidate_index];
    let new = (old + delta).clamp(0.0, 100.0);
    let actual = new - old;
    let others_total = (100.0 - old).max(0.0);
    shares[candidate_index] = new;

    if actual > 0.0 {
        if others_total > f64::EPSILON {
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share -= actual * *share / others_total;
                }
            }
        }
    } else if actual < 0.0 {
        if others_total > f64::EPSILON {
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share += (-actual) * *share / others_total;
                }
            }
        } else {
            let each = (-actual) / (shares.len() - 1) as f64;
            for (index, share) in shares.iter_mut().enumerate() {
                if index != candidate_index {
                    *share += each;
                }
            }
        }
    }
    normalize(shares);
}

fn transfer_share(shares: &mut [f64], from: usize, to: usize, requested: f64) {
    if from >= shares.len() || to >= shares.len() || from == to || requested <= 0.0 {
        return;
    }
    let amount = requested.min(shares[from]);
    shares[from] -= amount;
    shares[to] += amount;
    normalize(shares);
}

fn normalize(shares: &mut [f64]) {
    let total: f64 = shares.iter().sum();
    if total <= f64::EPSILON {
        return;
    }
    for share in shares {
        *share = (*share / total * 100.0).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_spend_budget_and_receive_weekly_funding() {
        let mut game = GameState::new(2024, Side::Incumbent, 123);
        let before = game.budget;
        let outcome = game.apply_action(ActionId::PolicyDelivery).unwrap();
        assert_eq!(game.budget, before - 20 + WEEKLY_FUNDING);
        assert_eq!(outcome.round, 1);
        assert!(game.polls.iter().all(|share| *share >= 0.0));
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
    }

    #[test]
    fn challenger_can_use_legal_assembly_and_grow_turnout() {
        let mut game = GameState::new(2016, Side::Challenger, 7);
        let before = game.turnout_momentum;
        game.apply_action(ActionId::LegalAssembly).unwrap();
        assert!(game.turnout_momentum > before);
    }

    #[test]
    fn same_seed_produces_the_same_campaign_and_final_draw() {
        let mut left = GameState::new(2024, Side::Challenger, 42);
        let mut right = GameState::new(2024, Side::Challenger, 42);
        for id in [
            ActionId::PolicyForum,
            ActionId::LegalAssembly,
            ActionId::CommunityTour,
        ] {
            left.apply_action(id).unwrap();
            right.apply_action(id).unwrap();
        }
        assert_eq!(left.polls, right.polls);
        assert_eq!(left.finish().shares, right.finish().shares);
    }

    #[test]
    fn twelve_round_limit_is_enforced() {
        let mut game = GameState::new(2020, Side::Challenger, 1);
        for _ in 0..CAMPAIGN_ROUNDS {
            let action = actions_for(Side::Challenger)[0];
            game.apply_action(action.id).unwrap();
        }
        assert!(game.is_finished());
        assert!(game.apply_action(ActionId::CommunityTour).is_err());
    }

    #[test]
    fn resting_recovers_stamina_without_poll_swing() {
        let mut game = GameState::new(2024, Side::Challenger, 99);
        game.stamina = 40.0;
        let polls_before = game.polls.clone();
        let result = game.apply_action(ActionId::RestAndPrepare).unwrap();
        assert_eq!(game.polls, polls_before);
        assert_eq!(result.stamina_after, 62.0);
    }

    #[test]
    fn local_scenario_uses_region_context_and_three_party_polls() {
        let region = crate::history::find_region("臺北市").unwrap();
        let game = GameState::new_local_with_style(
            2026,
            Side::Challenger,
            15,
            CampaignStyle::Organizer,
            region,
        );
        assert_eq!(game.region.unwrap().name, "臺北市");
        assert_eq!(game.candidates.len(), 3);
        assert_eq!(game.player_index, 1);
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
    }

    #[test]
    fn multi_candidate_poll_coordination_produces_a_report() {
        let region = crate::history::find_region("新竹市").unwrap();
        let mut game = GameState::new_local_with_style(
            2026,
            Side::Challenger,
            13,
            CampaignStyle::Communicator,
            region,
        );
        let outcome = game.apply_action(ActionId::PollCoordination).unwrap();
        assert!(outcome.coordination_note.is_some());
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
    }
}
