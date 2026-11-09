use project_twn::campaign::{CampaignDifficulty, CampaignSetup, EventPack, EventTimeline};
use project_twn::election_night::count_by_region;
use project_twn::electoral::{
    allocate_party_list_seats, party_ticket_support, PartyTicket, PARTY_LIST_SEATS,
    PARTY_LIST_THRESHOLD_PCT,
};
use project_twn::game_content::{
    find_party, find_region, PartyProfile, RegionProfile, ELECTIONS, EVENTS, LEGISLATURE_SNAPSHOTS,
    PARTY_PROFILES, REGIONS, VOTER_SEGMENTS,
};
use project_twn::governance::issue_label;
use project_twn::primary::{PrimaryChoice, PrimaryContest, PRIMARY_ROUNDS};
use project_twn::recall::{
    RecallActionId, RecallEnding, RecallPhase, RecallSide, RecallState, RecallTemplate,
    SIGNATURE_REVIEW_CHECKS,
};
use project_twn::roster::{CandidateRoster, RosterEntry};
use project_twn::savegame::{
    CampaignSaveConfig, RecallSaveConfig, SaveCommand, SaveConfig, SaveGame,
};
use project_twn::sim::{
    ActionId, CampaignScorecard, CampaignStyle, ElectionOutcome, GameError, GameState,
    ModelParameters, SensitivityParameter, Side, StafferOutcome, StoryOutcome, TacticCategory,
    CAMPAIGN_ROUNDS, DAYS_PER_WEEK,
};
use project_twn::staffer::{
    character_memory_echo as staffer_character_memory_echo,
    continuity_bonus as staffer_continuity_bonus, daily_route_echo as staffer_daily_route_echo,
    daily_scene as staffer_daily_scene,
};
use project_twn::story::{
    coherence_bonus, daily_route_echo as candidate_daily_route_echo,
    ending_note as candidate_story_ending_note,
};
use project_twn::strategy::{
    StrategyAxis, StrategyEngine, StrategyProfile, StrategyRecommendation,
};
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const CAMPAIGN_LOG_HEADER: &str = "year\tmode\tregion\trole\tstyle\tstrategy\tseed\twinner\tplayer_win_probability\tobjectives_completed\tpolicy\ttrust\tmobilization\tevent_response\tresource_management\tversatility\texposed_tactics\tsanctioned_funds\tdifficulty\tevent_pack\ttimeline\tstory_path\tstaffer_path";
const BATCH_LOG_HEADER: &str = "year\tmode\tregion\trole\tstyle\tstrategy\tseed\tsensitivity_parameter\tmultiplier\twinner_candidate\twinner_party\tplayer_party\tplayer_won\tplayer_win_probability\tplayer_final_share\tsigned_margin\tobjectives_completed\tbudget_remaining\tstamina_remaining\ttrust\tmobilization\tpolicy\tevent_response\tresource_management\tversatility\texposed_tactics\tsanctioned_funds\tsensitivity_parameter_2\tmultiplier_2\tpositive_actions\tneutral_actions\tnegative_actions\tmodule_support_mean\tmodule_context_mean\tmodule_trust_mean\tmodule_mobilization_mean\tmodule_objective_mean\tmodule_sustainability_mean\tmodule_integrity_mean\tmodule_infrastructure_mean\taction_composition\tdifficulty\tevent_pack\ttimeline\tstory_path";
const MAX_BATCH_RUNS: u32 = 10_000;
const MAX_STRATEGY_SWEEP_RUNS: u32 = 1_000;
const MAX_SENSITIVITY_SEEDS: u32 = 1_000;
const MAX_INTERACTION_SENSITIVITY_SEEDS: u32 = 100;

struct Options {
    year: u16,
    role: Option<Side>,
    style: CampaignStyle,
    strategy: StrategyProfile,
    seed: u64,
    auto: bool,
    show_scenarios: bool,
    list_regions: bool,
    list_parties: bool,
    region: Option<String>,
    party: Option<String>,
    setup: CampaignSetup,
    primary: bool,
    staffer_mode: bool,
    recall_mode: bool,
    recall_side: Option<RecallSide>,
    recall_template: RecallTemplate,
    recall_template_selected: bool,
    log: Option<String>,
    batch: Option<u32>,
    sweep_styles: bool,
    sweep_strategies: bool,
    sensitivity: bool,
    interaction_sensitivity: bool,
    batch_log: Option<String>,
    roster: Option<String>,
    roster_editor: Option<String>,
    save_path: Option<String>,
    load_path: Option<String>,
    roster_data: Option<CandidateRoster>,
}

#[derive(Clone, Copy)]
struct BatchScenario<'a> {
    options: &'a Options,
    role: Side,
    style: CampaignStyle,
    region: Option<&'static RegionProfile>,
    selected_party: Option<&'static PartyProfile>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ActionMenuCategory {
    PolicyGovernance,
    GroundOrganization,
    CommunityCare,
    PublicCommunication,
    Coalition,
    HighRisk,
    Preparation,
}

impl ActionMenuCategory {
    const ALL: [Self; 7] = [
        Self::PolicyGovernance,
        Self::GroundOrganization,
        Self::CommunityCare,
        Self::PublicCommunication,
        Self::Coalition,
        Self::HighRisk,
        Self::Preparation,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::PolicyGovernance => "政策與治理",
            Self::GroundOrganization => "組織與街頭動員",
            Self::CommunityCare => "社區關懷與宗教行程",
            Self::PublicCommunication => "辯論、媒體與政見",
            Self::Coalition => "民調協調與跨黨整合",
            Self::HighRisk => "高風險攻防",
            Self::Preparation => "整備與休息",
        }
    }

    fn includes(self, id: ActionId) -> bool {
        match self {
            Self::PolicyGovernance => matches!(
                id,
                ActionId::PolicyDelivery
                    | ActionId::CrisisResponse
                    | ActionId::TransparencyReport
                    | ActionId::PublicServices
                    | ActionId::OversightPetition
            ),
            Self::GroundOrganization => matches!(
                id,
                ActionId::CommunityTour
                    | ActionId::LegalAssembly
                    | ActionId::CampaignHeadquarters
                    | ActionId::VolunteerCanvassing
                    | ActionId::GroundTeamLeaflets
                    | ActionId::StreetSweep
                    | ActionId::TurnoutMobilization
                    | ActionId::SupportersAssociation
            ),
            Self::CommunityCare => matches!(
                id,
                ActionId::ReligiousCommunityVisit
                    | ActionId::PilgrimageParticipation
                    | ActionId::MassAttendance
                    | ActionId::VulnerableCommunityCare
                    | ActionId::DisasterResponseVisit
            ),
            Self::PublicCommunication => matches!(
                id,
                ActionId::PolicyForum
                    | ActionId::PublicMessage
                    | ActionId::Debate
                    | ActionId::MediaInterview
                    | ActionId::PolicyPresentation
                    | ActionId::FactCheckResponse
            ),
            Self::Coalition => matches!(id, ActionId::CoalitionTalks | ActionId::PollCoordination),
            Self::HighRisk => matches!(
                id,
                ActionId::FabricatedPoll
                    | ActionId::AnonymousSmear
                    | ActionId::CoordinatedOpinionWar
            ),
            Self::Preparation => id == ActionId::RestAndPrepare,
        }
    }
}

fn main() {
    if let Err(message) = run() {
        eprintln!("錯誤：{message}\n");
        print_usage();
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let mut options = parse_args()?;
    let load_path = options.load_path.clone();
    let loaded_save = match load_path.as_deref() {
        Some(path) => {
            let save = SaveGame::read(path)?;
            apply_saved_options(&mut options, &save)?;
            if options.save_path.is_none() {
                options.save_path = Some(path.to_string());
            }
            Some(save)
        }
        None => None,
    };
    if options.show_scenarios {
        print_scenarios();
        return Ok(());
    }
    if options.list_regions {
        print_regions();
        return Ok(());
    }
    if options.list_parties {
        print_parties();
        return Ok(());
    }
    if let Some(path) = options.roster_editor.as_deref() {
        run_roster_editor(path)?;
        return Ok(());
    }

    if options.recall_mode {
        if options.batch.is_some()
            || options.primary
            || options.staffer_mode
            || options.region.is_some()
            || options.party.is_some()
            || options.role.is_some()
            || options.log.is_some()
        {
            return Err(
                "罷免模式為獨立地方政治戰役，不能搭配批次、初選、幕僚競選或總統／地方選舉參數。"
                    .to_string(),
            );
        }
        if options.auto && options.save_path.is_some() {
            return Err("罷免自動示範尚未支援逐步存檔；請移除 --auto 或 --save。".to_string());
        }
        let side = match options.recall_side {
            Some(side) => side,
            None if options.auto => RecallSide::Initiator,
            None => ask_recall_side()?,
        };
        let template = if options.auto || options.recall_template_selected {
            options.recall_template
        } else {
            ask_recall_template()?
        };
        let mut game = RecallState::new(template, side, options.seed);
        let mut journal = loaded_save
            .as_ref()
            .map(|save| save.commands.clone())
            .unwrap_or_default();
        if loaded_save.is_some() {
            replay_recall(&mut game, &journal)?;
        }
        let save_config = RecallSaveConfig {
            side: side.key().to_string(),
            template: template.key().to_string(),
            seed: options.seed,
        };
        if let Some(path) = options.save_path.as_deref() {
            persist_recall(path, &save_config, &journal)?;
        }
        print_recall_intro(&game);
        if options.auto {
            run_recall_auto(&mut game);
        } else if game.phase != RecallPhase::Complete {
            run_recall_interactive(
                &mut game,
                &mut journal,
                options.save_path.as_deref(),
                &save_config,
            )?;
        }
        print_recall_result(&game);
        return Ok(());
    }

    if let Some(runs) = options.batch {
        if options.staffer_mode {
            return Err(
                "幕僚模式目前是單場劇情體驗，尚未接入批次模擬；請移除 --batch。".to_string(),
            );
        }
        let role = options
            .role
            .ok_or_else(|| "批次模擬必須明確指定 --role，避免把互動預設混入結果。".to_string())?;
        if options.primary {
            return Err("批次模式目前不支援 --primary；請先使用單場初選確認策略風格。".to_string());
        }
        if options.log.is_some() {
            return Err("批次結果請使用 --batch-log；--log 僅供單場紀錄。".to_string());
        }
        let (region, selected_party) = resolve_scenario(&options, role)?;
        return run_batch(&options, runs, role, region, selected_party);
    }
    if !options.recall_mode
        && (options.recall_side.is_some() || options.recall_template != RecallTemplate::Synthetic)
    {
        return Err("--recall-side 與 --recall-template 必須搭配 --recall。".to_string());
    }

    if options.auto && options.save_path.is_some() {
        return Err("自動示範模式尚未支援逐步存檔；請移除 --auto 或 --save。".to_string());
    }
    if options.staffer_mode && options.primary {
        return Err("幕僚模式由玩家扮演競選幕僚；目前不搭配 --primary 初選模式。".to_string());
    }
    let role = match options.role {
        Some(role) => role,
        None if options.auto || options.party.is_some() => Side::Challenger,
        None => ask_role()?,
    };
    let style = if options.primary {
        run_primary(options.seed, options.auto)?
    } else {
        options.style
    };
    let (region, selected_party) = resolve_scenario(&options, role)?;
    let scenario = BatchScenario {
        options: &options,
        role,
        style,
        region,
        selected_party,
    };
    let mut game = new_scenario_game(&scenario, options.seed, ModelParameters::default());
    if let Some(roster) = options.roster_data.as_ref() {
        roster.apply(&mut game)?;
    } else if let Some(path) = options.roster.as_deref() {
        let roster = CandidateRoster::read(path)?;
        roster.apply(&mut game)?;
        options.roster_data = Some(roster);
    }
    let save_config = make_campaign_save_config(&options, role, style, region, selected_party);
    let mut journal = loaded_save
        .as_ref()
        .map(|save| save.commands.clone())
        .unwrap_or_default();
    if loaded_save.is_some() {
        replay_campaign(&mut game, &journal)?;
    }
    if let Some(path) = options.save_path.as_deref() {
        persist_campaign(path, &save_config, &journal)?;
    }
    print_intro(&game, options.seed, options.strategy, options.staffer_mode);

    if options.auto {
        if options.staffer_mode {
            run_auto_staffer(&mut game, true, options.strategy);
        } else {
            run_auto(&mut game, true, options.strategy);
        }
    } else if !game.is_settled() && !game.is_finished() {
        run_interactive(
            &mut game,
            options.strategy,
            options.staffer_mode,
            &mut journal,
            options.save_path.as_deref(),
            &save_config,
        )?;
    }
    let result = print_final(&mut game);
    if options.staffer_mode {
        print_staffer_summary(&game);
    } else {
        print_candidate_story_summary(&game);
    }
    print_election_night(&game, &result, options.seed)?;
    if let Some(path) = options.log.as_deref() {
        save_campaign_log(
            path,
            &game,
            &result,
            options.seed,
            options.strategy,
            options.staffer_mode,
        )?;
        println!("本場紀錄已追加至：{path}");
    }
    Ok(())
}

fn print_election_night(
    game: &GameState,
    result: &ElectionOutcome,
    seed: u64,
) -> Result<(), String> {
    let regions = game
        .region
        .map(|region| vec![region])
        .unwrap_or_else(|| REGIONS.iter().collect());
    let candidate_names: Vec<_> = game
        .candidates
        .iter()
        .map(|candidate| candidate.name.clone())
        .collect();
    let parties: Vec<_> = game
        .candidates
        .iter()
        .map(|candidate| candidate.party.to_string())
        .collect();
    let count = count_by_region(&candidate_names, &parties, &result.shares, &regions, seed)
        .map_err(str::to_string)?;
    println!("\n【投票日開票｜虛構情境估算】分區票數會精確加總至本場終局模型票數；不是實際選舉資料。共揭曉 {} 個虛構地區：", count.regions.len());
    let mut running = vec![0_u64; count.candidates.len()];
    for (index, region) in count.regions.iter().enumerate() {
        for (candidate, votes) in running.iter_mut().zip(&region.candidate_votes) {
            *candidate += votes;
        }
        let region_winner = &count.candidates[region.winner_index];
        let mut leaders: Vec<_> = running.iter().enumerate().collect();
        leaders.sort_by(|(left_i, left), (right_i, right)| {
            right.cmp(left).then_with(|| left_i.cmp(right_i))
        });
        let leader = &count.candidates[leaders[0].0];
        let second = leaders.get(1).map(|(_, votes)| **votes).unwrap_or(0);
        println!(
            "  {:02}/{:02} {}｜投票率 {:>5.1}%｜{} 領先（{} 票）｜累計領先：{}（+{} 票）",
            index + 1,
            count.regions.len(),
            region.name,
            region.ballots as f64 / region.electorate as f64 * 100.0,
            region_winner,
            region.candidate_votes[region.winner_index],
            leader,
            leaders[0].1.saturating_sub(second)
        );
        let candidate_breakdown = count
            .candidates
            .iter()
            .zip(&region.candidate_votes)
            .map(|(candidate, votes)| {
                format!(
                    "{candidate} {votes} 票（{:.1}%）",
                    *votes as f64 / region.ballots as f64 * 100.0
                )
            })
            .collect::<Vec<_>>();
        println!("      本區票數：{}", candidate_breakdown.join("／"));
    }
    println!("全區合計：{} 張有效票。", count.total_valid_votes);
    let national_breakdown = count
        .candidates
        .iter()
        .zip(&count.national_votes)
        .map(|(candidate, votes)| {
            format!(
                "{candidate} {votes} 票（{:.1}%）",
                *votes as f64 / count.total_valid_votes as f64 * 100.0
            )
        })
        .collect::<Vec<_>>();
    println!("全國候選人總票數：{}", national_breakdown.join("／"));
    if game.region.is_none() {
        let candidate_inputs: Vec<_> = game
            .candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                (
                    candidate.party.to_string(),
                    candidate.name.clone(),
                    candidate.starting_share,
                    result.shares[index],
                )
            })
            .collect();
        let support = party_ticket_support(
            &candidate_inputs,
            game.candidates[game.player_index].party,
            game.trust,
            game.turnout_momentum,
        )
        .map_err(str::to_string)?;
        let tickets = allocate_party_list_seats(
            support,
            count.total_valid_votes,
            PARTY_LIST_SEATS,
            PARTY_LIST_THRESHOLD_PCT,
        )
        .map_err(str::to_string)?;
        println!("\n【全國政黨票／不分區席次情境】政黨支持度與候選人個人支持分開計算；政黨票經 5% 門檻後以最大餘數法分配 34 席：");
        for ticket in tickets {
            println!(
                "  {}｜政黨支持度 {:>5.2}%｜政黨票 {:>7} 張（{:>5.2}%）｜不分區席次 {}",
                ticket.party, ticket.support_pct, ticket.votes, ticket.vote_share_pct, ticket.seats
            );
        }
    }
    Ok(())
}

fn apply_saved_options(options: &mut Options, save: &SaveGame) -> Result<(), String> {
    options.auto = false;
    match &save.config {
        SaveConfig::Campaign(config) => {
            options.year = config.year;
            options.role = Some(parse_role(&config.side)?);
            options.style = parse_style(&config.style)?;
            options.strategy = StrategyProfile::parse(&config.strategy)?;
            options.seed = config.seed;
            options.region = config.region.clone();
            options.party = config.party.clone();
            options.setup.difficulty = CampaignDifficulty::parse(&config.difficulty)
                .ok_or_else(|| "存檔難度設定不支援。".to_string())?;
            options.setup.event_pack = EventPack::parse(&config.event_pack)
                .ok_or_else(|| "存檔事件包設定不支援。".to_string())?;
            options.setup.timeline = EventTimeline::parse(&config.timeline)
                .ok_or_else(|| "存檔事件時序設定不支援。".to_string())?;
            options.staffer_mode = config.staffer;
            options.recall_mode = false;
            options.recall_side = None;
            options.roster_data = config.roster.clone();
        }
        SaveConfig::Recall(config) => {
            options.recall_mode = true;
            options.recall_side = Some(parse_recall_side(&config.side)?);
            options.recall_template = RecallTemplate::parse(&config.template)
                .ok_or_else(|| "存檔罷免情境設定不支援。".to_string())?;
            options.recall_template_selected = true;
            options.seed = config.seed;
            options.role = None;
            options.region = None;
            options.party = None;
            options.staffer_mode = false;
            options.primary = false;
            options.roster_data = None;
        }
    }
    Ok(())
}

fn make_campaign_save_config(
    options: &Options,
    role: Side,
    style: CampaignStyle,
    region: Option<&RegionProfile>,
    party: Option<&PartyProfile>,
) -> CampaignSaveConfig {
    CampaignSaveConfig {
        year: options.year,
        side: match role {
            Side::Incumbent => "incumbent",
            Side::Challenger => "challenger",
        }
        .to_string(),
        style: style_key(style).to_string(),
        strategy: options.strategy.key().to_string(),
        seed: options.seed,
        region: region.map(|region| region.key.to_string()),
        party: party.map(|party| party.name.to_string()),
        difficulty: options.setup.difficulty.key().to_string(),
        event_pack: options.setup.event_pack.key().to_string(),
        timeline: options.setup.timeline.key().to_string(),
        staffer: options.staffer_mode,
        roster: options.roster_data.clone(),
    }
}

fn style_key(style: CampaignStyle) -> &'static str {
    match style {
        CampaignStyle::Balanced => "balanced",
        CampaignStyle::Policy => "policy",
        CampaignStyle::Organizer => "organizer",
        CampaignStyle::Communicator => "communicator",
    }
}

fn replay_campaign(game: &mut GameState, commands: &[SaveCommand]) -> Result<(), String> {
    for (index, command) in commands.iter().enumerate() {
        let result = match command {
            SaveCommand::Action(action) => ActionId::ALL
                .get(*action)
                .ok_or_else(|| "行動索引錯誤".to_string())
                .and_then(|id| {
                    game.apply_action(*id)
                        .map(|_| ())
                        .map_err(|error| error.to_string())
                }),
            SaveCommand::StoryChoice(choice) => game
                .apply_story_choice(*choice)
                .map(|_| ())
                .map_err(|error| error.to_string()),
            SaveCommand::SkipStory => game.skip_story_beat().map_err(|error| error.to_string()),
            SaveCommand::StafferChoice(choice) => game
                .apply_staffer_choice(*choice)
                .map(|_| ())
                .map_err(|error| error.to_string()),
            SaveCommand::SkipStaffer => {
                game.skip_staffer_brief().map_err(|error| error.to_string())
            }
            SaveCommand::SettleEarly => {
                game.finish();
                Ok(())
            }
            SaveCommand::RecallAction(_) => Err("罷免行動不可用於競選存檔".to_string()),
        };
        result.map_err(|error| format!("無法重播存檔第 {} 個決策：{error}", index + 1))?;
    }
    game.validate()
        .map_err(|error| format!("存檔狀態檢查失敗：{error}"))
}

fn replay_recall(game: &mut RecallState, commands: &[SaveCommand]) -> Result<(), String> {
    for (index, command) in commands.iter().enumerate() {
        let SaveCommand::RecallAction(action) = command else {
            return Err("罷免存檔含有不相容的競選決策。".to_string());
        };
        let action = RecallActionId::ALL
            .get(*action)
            .ok_or_else(|| "存檔罷免行動索引錯誤。".to_string())?;
        game.apply_action(*action)
            .map_err(|error| format!("無法重播罷免存檔第 {} 個決策：{error}", index + 1))?;
    }
    Ok(())
}

fn persist_campaign(
    path: &str,
    config: &CampaignSaveConfig,
    commands: &[SaveCommand],
) -> Result<(), String> {
    SaveGame {
        config: SaveConfig::Campaign(config.clone()),
        commands: commands.to_vec(),
    }
    .write(path)
}

fn persist_recall(
    path: &str,
    config: &RecallSaveConfig,
    commands: &[SaveCommand],
) -> Result<(), String> {
    SaveGame {
        config: SaveConfig::Recall(config.clone()),
        commands: commands.to_vec(),
    }
    .write(path)
}

fn checkpoint_campaign(
    game: &GameState,
    path: Option<&str>,
    config: &CampaignSaveConfig,
    commands: &[SaveCommand],
) -> Result<(), String> {
    if let Some(path) = path {
        game.validate()
            .map_err(|error| format!("存檔前狀態檢查失敗：{error}"))?;
        persist_campaign(path, config, commands)?;
    }
    Ok(())
}

fn checkpoint_recall(
    path: Option<&str>,
    config: &RecallSaveConfig,
    commands: &[SaveCommand],
) -> Result<(), String> {
    if let Some(path) = path {
        persist_recall(path, config, commands)?;
    }
    Ok(())
}

fn resolve_scenario(
    options: &Options,
    role: Side,
) -> Result<
    (
        Option<&'static RegionProfile>,
        Option<&'static PartyProfile>,
    ),
    String,
> {
    let region = match options.region.as_deref() {
        Some(query) => Some(
            find_region(query)
                .ok_or_else(|| format!("找不到地方情境：{query}。使用 --regions 檢視可選清單。"))?,
        ),
        None => None,
    };
    if region.is_some() && options.year != 2026 {
        return Err("地方選舉模式目前以 2026 年情境進行，請使用 --year 2026。".to_string());
    }
    let selected_party = match options.party.as_deref() {
        Some(query) => Some(
            find_party(query)
                .ok_or_else(|| format!("找不到政黨情境範本：{query}。使用 --parties 檢視清單。"))?,
        ),
        None => None,
    };
    if selected_party.is_some() && region.is_none() {
        return Err("--party 只適用於搭配 --region 的地方情境。".to_string());
    }
    if let (Some(party), Some(region)) = (selected_party, region) {
        let previous_party = find_party(region.previous_winner_party)
            .map(|profile| profile.name)
            .unwrap_or(region.previous_winner_party);
        if role == Side::Incumbent && party.name != previous_party {
            return Err("執政方 --party 必須符合該虛構地區的上一輪情境執政黨；若扮演其他政黨請選 --role challenger。".to_string());
        }
    }
    Ok((region, selected_party))
}

fn new_scenario_game(
    scenario: &BatchScenario<'_>,
    seed: u64,
    model_parameters: ModelParameters,
) -> GameState {
    match scenario.region {
        Some(region) => GameState::new_local_with_party_and_setup(
            scenario.options.year,
            scenario.role,
            seed,
            scenario.style,
            region,
            scenario.selected_party,
            model_parameters,
            scenario.options.setup,
        ),
        None => GameState::new_with_setup(
            scenario.options.year,
            scenario.role,
            seed,
            scenario.style,
            model_parameters,
            scenario.options.setup,
        ),
    }
}

fn run_roster_editor(path: &str) -> Result<(), String> {
    println!(
        "【虛構候選人名冊編輯器】名冊會獨立存成 TSV；可編輯姓名、虛構政黨、起始盤與玩家候選人。\n"
    );
    println!(
        "可用虛構政黨：{}",
        PARTY_PROFILES
            .iter()
            .map(|party| party.name)
            .collect::<Vec<_>>()
            .join("、")
    );
    let count: usize = loop {
        let value = prompt_line("候選人人數（2–5）：")?;
        if let Ok(value) = value.parse::<usize>() {
            if (2..=5).contains(&value) {
                break value;
            }
        }
        println!("請輸入 2 至 5 的整數。\n");
    };
    let mut candidates = Vec::with_capacity(count);
    for index in 0..count {
        println!("\n候選人 {}：", index + 1);
        let name = prompt_line("  姓名：")?;
        let party = loop {
            let value = prompt_line("  政黨名稱（須使用上述虛構政黨）：")?;
            if find_party(&value).is_some() {
                break value;
            }
            println!("  這個政黨不在虛構名冊中，請重新輸入。\n");
        };
        let starting_support: f64 = loop {
            let value = prompt_line("  起始支持度（0–100）：")?;
            if let Ok(value) = value.parse::<f64>() {
                if value.is_finite() && (0.0..=100.0).contains(&value) {
                    break value;
                }
            }
            println!("  請輸入 0 至 100 的數字。\n");
        };
        let player = loop {
            match prompt_line("  本場玩家候選人？（y/n）：")?
                .to_ascii_lowercase()
                .as_str()
            {
                "y" | "yes" | "是" => break true,
                "n" | "no" | "否" => break false,
                _ => println!("  請輸入 y 或 n。"),
            }
        };
        candidates.push(RosterEntry {
            name,
            party,
            starting_support,
            player,
        });
    }
    let roster = CandidateRoster { candidates };
    roster.validate()?;
    roster.write(path)?;
    println!("\n名冊已儲存：{path}\n使用方式：project-twn --year 2024 --role challenger --roster \"{path}\"");
    Ok(())
}

fn prompt_line(label: &str) -> Result<String, String> {
    print!("{label}");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut value = String::new();
    io::stdin()
        .read_line(&mut value)
        .map_err(|error| error.to_string())?;
    Ok(value.trim().to_string())
}

fn parse_args() -> Result<Options, String> {
    let mut args = env::args().skip(1);
    let mut options = Options {
        year: 2026,
        role: None,
        style: CampaignStyle::Balanced,
        strategy: StrategyProfile::Balanced,
        seed: 2026,
        auto: false,
        show_scenarios: false,
        list_regions: false,
        list_parties: false,
        region: None,
        party: None,
        setup: CampaignSetup::default(),
        primary: false,
        staffer_mode: false,
        recall_mode: false,
        recall_side: None,
        recall_template: RecallTemplate::Synthetic,
        recall_template_selected: false,
        log: None,
        batch: None,
        sweep_styles: false,
        sweep_strategies: false,
        sensitivity: false,
        interaction_sensitivity: false,
        batch_log: None,
        roster: None,
        roster_editor: None,
        save_path: None,
        load_path: None,
        roster_data: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--history" | "--scenarios" => options.show_scenarios = true,
            "--regions" => options.list_regions = true,
            "--parties" => options.list_parties = true,
            "--auto" | "--demo" => options.auto = true,
            "--primary" => options.primary = true,
            "--staffer" => options.staffer_mode = true,
            "--recall" => options.recall_mode = true,
            "--recall-side" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--recall-side 後面需要 initiator 或 opposition".to_string())?;
                options.recall_side = Some(parse_recall_side(&value)?);
            }
            "--recall-template" => {
                let value = args.next().ok_or_else(|| {
                    "--recall-template 後面需要 fictional、low-turnout、concentrated 或 high-mobilization".to_string()
                })?;
                options.recall_template = RecallTemplate::parse(&value).ok_or_else(|| {
                    format!("未知罷免情境盤型：{value}；可選 fictional、low-turnout、concentrated、high-mobilization")
                })?;
                options.recall_template_selected = true;
            }
            "--region" => {
                options.region = Some(
                    args.next()
                        .ok_or_else(|| "--region 後面需要縣市名稱或代碼".to_string())?,
                )
            }
            "--party" => {
                options.party = Some(
                    args.next()
                        .ok_or_else(|| "--party 後面需要政黨名稱或別名".to_string())?,
                )
            }
            "--difficulty" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--difficulty 後面需要 story、standard 或 hard".to_string())?;
                options.setup.difficulty = CampaignDifficulty::parse(&value)
                    .ok_or_else(|| format!("未知難度：{value}；可用 story、standard、hard"))?;
            }
            "--event-pack" => {
                let value = args.next().ok_or_else(|| {
                    "--event-pack 後面需要 all、democracy、cross-strait、local 或 media".to_string()
                })?;
                options.setup.event_pack =
                    EventPack::parse(&value).ok_or_else(|| format!("未知事件包：{value}"))?;
            }
            "--timeline" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--timeline 後面需要 weekly 或 date".to_string())?;
                options.setup.timeline = EventTimeline::parse(&value)
                    .ok_or_else(|| format!("未知事件排程：{value}；可用 weekly、date"))?;
            }
            "--log" => {
                options.log = Some(
                    args.next()
                        .ok_or_else(|| "--log 後面需要 TSV 檔案路徑".to_string())?,
                )
            }
            "--batch" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--batch 後面需要批次場數".to_string())?;
                options.batch = Some(
                    value
                        .parse()
                        .map_err(|_| "批次場數必須是正整數".to_string())?,
                );
            }
            "--sweep-styles" => options.sweep_styles = true,
            "--sweep-strategies" => options.sweep_strategies = true,
            "--sensitivity" => options.sensitivity = true,
            "--interaction-sensitivity" => options.interaction_sensitivity = true,
            "--batch-log" => {
                options.batch_log = Some(
                    args.next()
                        .ok_or_else(|| "--batch-log 後面需要 TSV 檔案路徑".to_string())?,
                )
            }
            "--roster" => {
                options.roster = Some(
                    args.next()
                        .ok_or_else(|| "--roster 後面需要 TSV 檔案路徑".to_string())?,
                )
            }
            "--roster-editor" => {
                options.roster_editor = Some(
                    args.next()
                        .ok_or_else(|| "--roster-editor 後面需要 TSV 檔案路徑".to_string())?,
                )
            }
            "--save" => {
                options.save_path = Some(
                    args.next()
                        .ok_or_else(|| "--save 後面需要存檔路徑".to_string())?,
                )
            }
            "--load" => {
                options.load_path = Some(
                    args.next()
                        .ok_or_else(|| "--load 後面需要存檔路徑".to_string())?,
                )
            }
            "--simulate" => {}
            "--year" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--year 後面需要年份".to_string())?;
                options.year = value.parse().map_err(|_| "年份必須是數字".to_string())?;
            }
            "--role" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--role 後面需要 incumbent 或 challenger".to_string())?;
                options.role = Some(parse_role(&value)?);
            }
            "--style" => {
                let value = args.next().ok_or_else(|| {
                    "--style 後面需要 balanced、policy、organizer 或 communicator".to_string()
                })?;
                options.style = parse_style(&value)?;
            }
            "--strategy" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--strategy 後面需要策略名稱".to_string())?;
                options.strategy = StrategyProfile::parse(&value)?;
            }
            "--seed" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--seed 後面需要整數".to_string())?;
                options.seed = value
                    .parse()
                    .map_err(|_| "seed 必須是無號整數".to_string())?;
            }
            unexpected if !unexpected.starts_with('-') => {
                options.year = unexpected
                    .parse()
                    .map_err(|_| format!("無法辨識的參數：{unexpected}"))?;
            }
            unknown => return Err(format!("不支援的參數：{unknown}")),
        }
    }
    if !(1996..=2026).contains(&options.year) {
        return Err("情境年份請選 1996–2026".to_string());
    }
    if let Some(runs) = options.batch {
        if runs == 0 || runs > MAX_BATCH_RUNS {
            return Err(format!("批次場數需介於 1–{MAX_BATCH_RUNS}。"));
        }
    }
    if options.sweep_styles && options.batch.is_none() {
        return Err("--sweep-styles 必須搭配 --batch 使用。".to_string());
    }
    if options.sweep_strategies && options.batch.is_none() {
        return Err("--sweep-strategies 必須搭配 --batch 使用。".to_string());
    }
    if options.sweep_strategies && options.sweep_styles {
        return Err("--sweep-strategies 與 --sweep-styles 請分開執行。".to_string());
    }
    if options.sweep_strategies
        && options
            .batch
            .is_some_and(|runs| runs > MAX_STRATEGY_SWEEP_RUNS)
    {
        return Err(format!(
            "策略比較每個策略最多使用 {MAX_STRATEGY_SWEEP_RUNS} 個 seed。"
        ));
    }
    if options.sensitivity && options.batch.is_none() {
        return Err("--sensitivity 必須搭配 --batch 使用。".to_string());
    }
    if options.sensitivity && (options.sweep_styles || options.sweep_strategies) {
        return Err("--sensitivity 請與 --sweep-styles、--sweep-strategies 分開執行。".to_string());
    }
    if options.interaction_sensitivity && options.batch.is_none() {
        return Err("--interaction-sensitivity 必須搭配 --batch 使用。".to_string());
    }
    if options.interaction_sensitivity
        && (options.sweep_styles || options.sweep_strategies || options.sensitivity)
    {
        return Err("--interaction-sensitivity 請與其他批次比較模式分開執行。".to_string());
    }
    if options.interaction_sensitivity
        && options
            .batch
            .is_some_and(|runs| runs > MAX_INTERACTION_SENSITIVITY_SEEDS)
    {
        return Err(format!(
            "二因子交互作用敏感度每個情境最多使用 {MAX_INTERACTION_SENSITIVITY_SEEDS} 個 seed。"
        ));
    }
    if options.sensitivity
        && options
            .batch
            .is_some_and(|runs| runs > MAX_SENSITIVITY_SEEDS)
    {
        return Err(format!(
            "單因子敏感度每個參數最多使用 {MAX_SENSITIVITY_SEEDS} 個 seed。"
        ));
    }
    if options.batch_log.is_some() && options.batch.is_none() {
        return Err("--batch-log 必須搭配 --batch 使用。".to_string());
    }
    if options.roster.is_some() && options.batch.is_some() {
        return Err("自訂候選人名冊目前限單場戰役，不能搭配 --batch。".to_string());
    }
    if options.roster.is_some() && options.recall_mode {
        return Err("候選人名冊不適用於獨立罷免模式。".to_string());
    }
    if options.roster.is_some() && options.roster_editor.is_some() {
        return Err("--roster 與 --roster-editor 請分開使用。".to_string());
    }
    if options.save_path.is_some() && options.batch.is_some() {
        return Err("存檔／讀檔目前限互動式單場戰役，不支援批次模式。".to_string());
    }
    if options.save_path.is_some() && options.auto {
        return Err("逐步存檔目前限互動模式；請移除 --auto。".to_string());
    }
    if options.load_path.is_some()
        && (options.show_scenarios
            || options.list_regions
            || options.list_parties
            || options.roster_editor.is_some())
    {
        return Err("--load 請單獨用於載入戰役，不可搭配清單或名冊編輯器。".to_string());
    }
    Ok(options)
}

fn parse_role(value: &str) -> Result<Side, String> {
    match value.to_ascii_lowercase().as_str() {
        "incumbent" | "inc" | "執政" | "連任" => Ok(Side::Incumbent),
        "challenger" | "opp" | "在野" | "挑戰" => Ok(Side::Challenger),
        _ => Err(format!("角色請用 incumbent 或 challenger，收到：{value}")),
    }
}

fn parse_style(value: &str) -> Result<CampaignStyle, String> {
    match value.to_ascii_lowercase().as_str() {
        "balanced" | "均衡" => Ok(CampaignStyle::Balanced),
        "policy" | "政策" => Ok(CampaignStyle::Policy),
        "organizer" | "組織" => Ok(CampaignStyle::Organizer),
        "communicator" | "溝通" => Ok(CampaignStyle::Communicator),
        _ => Err(format!(
            "風格請用 balanced、policy、organizer 或 communicator，收到：{value}"
        )),
    }
}

fn ask_role() -> Result<Side, String> {
    loop {
        println!("此處選的是執政／在野角色；玩家候選人會依年份與地區基準自動指派，開場會列出姓名或政黨情境代表。");
        println!("選擇陣營：1) 執政／尋求連任  2) 在野挑戰");
        print!("請輸入 1 或 2：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        match input.trim() {
            "1" => return Ok(Side::Incumbent),
            "2" => return Ok(Side::Challenger),
            _ => println!("請輸入 1 或 2。"),
        }
    }
}

fn parse_recall_side(value: &str) -> Result<RecallSide, String> {
    match value.to_ascii_lowercase().as_str() {
        "initiator" | "support" | "yes" | "罷免" | "提議" => Ok(RecallSide::Initiator),
        "opposition" | "oppose" | "no" | "反對" | "反罷免" => Ok(RecallSide::Opposition),
        _ => Err(format!(
            "罷免陣營請用 initiator 或 opposition，收到：{value}"
        )),
    }
}

fn ask_recall_side() -> Result<RecallSide, String> {
    loop {
        println!("玩家扮演人民發動罷免的提議／連署方，或反對罷免方；本模式不包含彈劾。");
        println!("選擇陣營：1) 罷免提議／連署方  2) 反對罷免方");
        print!("請輸入 1 或 2：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        match input.trim() {
            "1" => return Ok(RecallSide::Initiator),
            "2" => return Ok(RecallSide::Opposition),
            _ => println!("請輸入 1 或 2。"),
        }
    }
}

fn ask_recall_template() -> Result<RecallTemplate, String> {
    loop {
        println!("\n選擇罷免目標與選區盤型（皆為虛構情境）：");
        for (index, template) in RecallTemplate::ALL.iter().copied().enumerate() {
            println!(
                "  {}) {}｜{}（{}）｜焦點：{}",
                index + 1,
                template.label(),
                template.target_name(),
                template.district_name(),
                template.constituency_issue()
            );
        }
        print!("請輸入 1–4：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        if let Ok(selection) = input.trim().parse::<usize>() {
            if let Some(index) = selection.checked_sub(1) {
                if let Some(template) = RecallTemplate::ALL.get(index) {
                    return Ok(*template);
                }
            }
        }
        println!("請輸入 1–4 其中一個選項。\n");
    }
}

fn print_recall_intro(game: &RecallState) {
    println!("\n=== 人民罷免戰役｜Project-TWN ===");
    println!(
        "虛構罷免目標：區域立法委員 {}（{}）　｜虛構選區：{}\n情境盤型：{}　玩家陣營：{}　選舉人：{}　隨機種子：固定可重現",
        game.template.target_name(),
        game.template.target_party(),
        game.template.district_name(),
        game.template.label(),
        game.player_side.label(),
        game.electorate
    );
    println!("{}", game.template.briefing());
    let eligibility = game.template.eligibility();
    println!(
        "對象資格查核：{}，就職 {} 個月；{}。",
        match eligibility.office {
            project_twn::recall::RecallOffice::ConstituencyLegislator => "區域立法委員",
            project_twn::recall::RecallOffice::PartyListLegislator => "不分區立法委員",
        },
        eligibility.months_in_office,
        if game.target_is_eligible() {
            "已滿一年且符合本作可罷免對象條件"
        } else {
            "不符本作可罷免對象條件"
        }
    );
    println!(
        "選區議題：{}　｜目標公共服務紀錄：{:.1}/100（遊戲狀態，會隨答辯或查證改變）",
        game.template.constituency_issue(),
        game.target_record
    );
    println!("僅模擬人民依法發動的罷免；不納入彈劾。所有人物、選區與盤勢均為虛構情境，不是歷史個案重演、實際民調或因果結論。");
    println!(
        "法定門檻：提議 {} 人（1%）→有效連署 {} 人（10%）→同意票至少 {} 人（1/4），且多於不同意票。",
        game.proposal_threshold, game.signature_threshold, game.yes_floor
    );
    println!("時間視窗（遊戲壓縮）：提議準備 14 日、連署 60 日；提議或連署經查對後如接近門檻但仍不足，各可開啟一次補提窗口（最長 10 日）；成案後投票宣傳 28 日。每次選擇推進一個策略區塊。");
    println!("行政查對項目（遊戲簡化呈現）：身分核對、罷免資格、重複簽署、名冊完整性；補提不會免除後續查對。");
    println!(
        "負面選項包含不實／死亡連署；不會成為有效連署，並可能帶來名冊查核及信任／預算懲處。\n"
    );
}

fn print_recall_status(game: &RecallState) {
    let phase_days = match game.phase {
        RecallPhase::Proposal => game.days_elapsed,
        RecallPhase::ProposalSupplement => game.days_elapsed.saturating_sub(14),
        RecallPhase::Signature => game
            .days_elapsed
            .saturating_sub(14 + if game.proposal_supplement_used { 10 } else { 0 }),
        RecallPhase::SignatureSupplement => game
            .days_elapsed
            .saturating_sub(74 + if game.proposal_supplement_used { 10 } else { 0 }),
        RecallPhase::VoteCampaign => game.days_elapsed.saturating_sub(
            74 + if game.proposal_supplement_used { 10 } else { 0 }
                + if game.supplement_used { 10 } else { 0 },
        ),
        RecallPhase::Complete => 0,
    };
    println!(
        "\n--- {}｜本階段第 {} / {} 日｜策略區塊 {}/{} ---",
        game.phase.label(),
        phase_days,
        game.current_days_limit(),
        game.blocks_elapsed,
        game.current_block_limit()
    );
    println!(
        "目標公共服務紀錄 {:.1}/100　｜罷免方信任 {:.1}　｜反罷免方信任 {:.1}",
        game.target_record, game.initiator_trust, game.opposition_trust
    );
    match game.phase {
        RecallPhase::Proposal => println!(
            "提議進度：{} / {} 人　｜　陣營信任：罷免方 {:.1}、反罷免方 {:.1}　｜　預算 {}",
            game.proposal_count,
            game.proposal_threshold,
            game.initiator_trust,
            game.opposition_trust,
            game.budget
        ),
        RecallPhase::ProposalSupplement => println!(
            "提議補提窗口：{} / {} 人；尚需 {} 人。已完成行政查對 {} 項（{}）；補提僅能使用一次、最長 10 日。",
            game.proposal_count,
            game.proposal_threshold,
            game.proposal_threshold.saturating_sub(game.proposal_count),
            game.administrative_checks_completed,
            SIGNATURE_REVIEW_CHECKS.join("、")
        ),
        RecallPhase::Signature => println!(
            "有效連署估計：{} / {} 人　｜表面總數：{}（含不實名冊 {}；不計入有效數）　｜階段結束查對：身分、資格、重複簽署、名冊完整性　｜查核壓力 {:.0}%　｜預算 {}",
            game.valid_signatures(),
            game.signature_threshold,
            game.raw_signature_count(),
            game.fraudulent_signatures,
            game.audit_pressure * 100.0,
            game.budget
        ),
        RecallPhase::SignatureSupplement => println!(
            "一次補提窗口：有效連署 {} / {} 人；尚需 {} 人。已完成行政查對 {} 項（{}）；補提僅能使用一次、最長 10 日，補提名冊仍須查對。",
            game.valid_signatures(),
            game.signature_threshold,
            game.signature_threshold.saturating_sub(game.valid_signatures()),
            game.administrative_checks_completed,
            SIGNATURE_REVIEW_CHECKS.join("、")
        ),
        RecallPhase::VoteCampaign => {
            let (yes, no) = game.estimated_vote_totals();
            println!(
                "罷免方意向 {:.1}%／動員 {:.1}　｜反罷免方意向 {:.1}%／動員 {:.1}　｜預估票數 同意 {}、不同意 {}　｜同意票門檻 {}　｜預算 {}",
                game.yes_intention_pct,
                game.yes_mobilization,
                game.no_intention_pct,
                game.no_mobilization,
                yes,
                no,
                game.yes_floor,
                game.budget
            );
        }
        RecallPhase::Complete => {}
    }
}

fn run_recall_interactive(
    game: &mut RecallState,
    journal: &mut Vec<SaveCommand>,
    save_path: Option<&str>,
    save_config: &RecallSaveConfig,
) -> Result<bool, String> {
    while game.phase != RecallPhase::Complete {
        print_recall_status(game);
        println!("\n【本階段行動】每次選擇代表一個跨日策略區塊，行動會觸發對手回應：");
        let choices = game.action_options();
        for (index, action) in choices.iter().enumerate() {
            let risk = if matches!(
                action.id,
                RecallActionId::FalseSignatures | RecallActionId::DeceasedSignatures
            ) {
                "｜違規高風險"
            } else {
                ""
            };
            println!(
                "  {}) {}｜預算 -{}{}\n     {}",
                index + 1,
                action.label,
                action.cost,
                risk,
                action.description
            );
        }
        print!("選行動（p 盤勢，? 說明，q 離開）：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        let value = input.trim();
        if value.eq_ignore_ascii_case("p") {
            print_recall_status(game);
            continue;
        }
        if value == "?" {
            println!("每階段提供少量情境相關選項；提議 14 日、連署 60 日、投票宣傳 28 日。成案及罷免成立都需分別通過法定人數／票數門檻；同意票支持度不等於有效簽名或實際出席。\n");
            continue;
        }
        if value.eq_ignore_ascii_case("q") {
            print!("離開目前罷免戰役？輸入 y 確認，其餘輸入返回：");
            io::stdout().flush().map_err(|error| error.to_string())?;
            let mut confirm = String::new();
            io::stdin()
                .read_line(&mut confirm)
                .map_err(|error| error.to_string())?;
            if confirm.trim().eq_ignore_ascii_case("y") {
                return Ok(false);
            }
            continue;
        }
        let Ok(selection) = value.parse::<usize>() else {
            println!("請輸入行動編號、p、? 或 q。\n");
            continue;
        };
        let Some(action) = choices.get(selection.saturating_sub(1)) else {
            println!("沒有這個行動選項。\n");
            continue;
        };
        match game.apply_action(action.id) {
            Ok(outcome) => {
                journal.push(SaveCommand::RecallAction(outcome.id.index()));
                checkpoint_recall(save_path, save_config, journal)?;
                println!("\n✓ {}｜{}", outcome.label, outcome.note);
                let report = outcome.report;
                match outcome.phase_before {
                    RecallPhase::Proposal | RecallPhase::ProposalSupplement => println!(
                        "本區塊變化（含對手回應）：提議人數 {} → {} / {}。",
                        report.proposal_before,
                        report.proposal_after,
                        game.proposal_threshold
                    ),
                    RecallPhase::Signature | RecallPhase::SignatureSupplement => println!(
                        "本區塊變化（含對手回應）：有效連署估計 {} → {} / {}；表面名冊仍以查對結果為準。",
                        report.valid_signatures_before,
                        report.valid_signatures_after,
                        game.signature_threshold
                    ),
                    RecallPhase::VoteCampaign => println!(
                        "本區塊變化（含對手回應）：罷免意向 {:+.2}、反罷免意向 {:+.2}；預估同意票 {} → {}、不同意票 {} → {}。",
                        report.yes_intention_after - report.yes_intention_before,
                        report.no_intention_after - report.no_intention_before,
                        report.yes_votes_before,
                        report.yes_votes_after,
                        report.no_votes_before,
                        report.no_votes_after
                    ),
                    RecallPhase::Complete => {}
                }
                println!(
                    "目標公共服務紀錄：{:.1} → {:.1}/100。",
                    report.target_record_before, report.target_record_after
                );
                println!(
                    "日程已推進至第 {} 日；預算餘額 {}。",
                    outcome.days_elapsed, outcome.budget_after
                );
            }
            Err(error) => println!("{error}\n"),
        }
    }
    Ok(true)
}

fn run_recall_auto(game: &mut RecallState) {
    while game.phase != RecallPhase::Complete {
        let action = match (game.player_side, game.phase) {
            (RecallSide::Initiator, RecallPhase::Proposal) => RecallActionId::BuildCoalition,
            (RecallSide::Initiator, RecallPhase::ProposalSupplement) => {
                RecallActionId::BuildCoalition
            }
            (RecallSide::Initiator, RecallPhase::Signature) if game.blocks_elapsed == 0 => {
                RecallActionId::BuildCoalition
            }
            (RecallSide::Initiator, RecallPhase::Signature) => RecallActionId::SignatureStations,
            (RecallSide::Initiator, RecallPhase::SignatureSupplement) => {
                RecallActionId::SignatureStations
            }
            (RecallSide::Initiator, RecallPhase::VoteCampaign) => RecallActionId::FieldMobilization,
            (RecallSide::Opposition, RecallPhase::Proposal) => RecallActionId::BuildCoalition,
            (RecallSide::Opposition, RecallPhase::ProposalSupplement) => {
                RecallActionId::PublicDefense
            }
            (RecallSide::Opposition, RecallPhase::Signature) => RecallActionId::LegalAudit,
            (RecallSide::Opposition, RecallPhase::SignatureSupplement) => {
                RecallActionId::LegalAudit
            }
            (RecallSide::Opposition, RecallPhase::VoteCampaign) => {
                RecallActionId::CounterMobilization
            }
            (_, RecallPhase::Complete) => break,
        };
        if game.apply_action(action).is_err() {
            break;
        }
    }
}

fn print_recall_result(game: &RecallState) {
    let Some(result) = &game.result else {
        println!("\n罷免戰役已離開，尚未結算。");
        return;
    };
    println!("\n=== 罷免程序結算 ===");
    match result.ending {
        RecallEnding::ProposalFailed => println!("未成案：提議人數未達門檻。"),
        RecallEnding::SignaturesFailed => println!("連署未成案：有效連署不足。"),
        RecallEnding::RecallPassed => println!("罷免通過：兩項投票門檻均達成。"),
        RecallEnding::RecallRejected => println!("罷免未通過：至少一項投票門檻未達成。"),
    }
    println!("原因：{}", result.reason);
    if matches!(
        result.ending,
        RecallEnding::RecallPassed | RecallEnding::RecallRejected
    ) {
        println!(
            "預估投票：同意 {}、不同意 {}、合計 {}；同意票門檻 {}。這是遊戲結算值，不是民調或預測。",
            result.yes_votes, result.no_votes, result.turnout, result.yes_floor
        );
    }
    println!(
        "程序檢視：有效連署 {} / {}；提議門檻 {}；罷免同意票須至少達原選區選舉人 1/4，且多於不同意票。",
        result.valid_signatures,
        result.signature_threshold,
        game.proposal_threshold
    );
    if game.fraud_attempts > 0 {
        println!(
            "不實名冊處理：嘗試 {} 次、表面數量 {}；{}。",
            game.fraud_attempts,
            game.fraudulent_signatures,
            if game.fraud_exposed {
                "查核揭露並受遊戲內懲處"
            } else {
                "違規名冊仍未計入有效連署"
            }
        );
    }
    if result.ending == RecallEnding::RecallPassed {
        println!(
            "後續程序：目標職務出缺後須另行依法辦理補選；席次不會自動判給提議方或任何政黨。本次罷免結果不預設補選勝者。"
        );
    }
}

fn print_intro(game: &GameState, seed: u64, strategy: StrategyProfile, staffer_mode: bool) {
    println!("\n=== 臺灣選戰模擬器 | Project-TWN ===");
    println!(
        "情境年份：{}　陣營：{}　能力風格：{}　策略引擎：{}　隨機種子：{}",
        game.campaign_year,
        game.side.label(),
        game.style.label(),
        strategy.label(),
        seed
    );
    println!(
        "難度：{}　事件包：{}　事件時序：{}",
        game.setup.difficulty.label(),
        game.setup.event_pack.label(),
        game.setup.timeline.label()
    );
    print_player_assignment(game);
    if staffer_mode {
        println!("遊戲模式：幕僚模式｜你是虛構競選團隊的幕僚，負責設定主軸、處理陳情與事件、協調地方網絡、檢視支持度並安排最後布局。每兩週一次幕僚會議，不占用每日行動，但會消耗部分競選資源；本模式以專屬情節取代一般模式支線，避免同時觸發重複陳情故事。\n");
    } else {
        println!(
            "遊戲模式：候選人模式｜你親自扮演候選人 {}，每天安排選戰行動，並在多週故事中作出取捨；行程敘事會回應已選路線與當日行動。\n",
            game.candidates[game.player_index].name
        );
    }
    println!("【快速開始】每天選一項行動並推進一天；先選分類，再選行動。輸入 r 執行推薦、? 查看說明，分類內輸入 d 編號先讀詳情。\n每日狀態會分開標示候選人支持度與政黨票支持度；執行後以 ✓【已選】標記，每週自動回顧。輸入 p 查看完整支持度分眾，q 提前結算時會再次確認。\n");
    if let Some(region) = game.region {
        println!(
            "地方情境：{}；上屆首長政黨為 {}。",
            region.name, region.previous_winner_party
        );
        println!(
            "地方關注：{}。初始三黨比例為遊戲情境參數，不是該地實際得票率或民調。",
            region.focus_label
        );
    } else {
        println!(
            "選舉基準：{} 年虛構總統情境；起始支持度是本作設定值，不是實際得票率或民調。",
            game.baseline_year
        );
    }
    if game.region.is_none() && game.campaign_year != game.baseline_year {
        println!("注意：本場使用政黨情境代表；起始支持度沿用最近一組虛構選戰設定，不代表當期民調或選舉結果。");
    }
    if game.campaign_year == 2026 {
        if game.region.is_some() {
            println!("2026 地方選舉模式：沿用虛構地方情境的上一輪執政聯盟與支持設定；本作投票日為 10-24。" );
        } else {
            println!(
                "2026 全國選戰情境：以 2024 年虛構選戰設定作為支持度基準；不是實際候選人或民調。"
            );
        }
    }
    println!("競選期：{CAMPAIGN_ROUNDS} 週 × {DAYS_PER_WEEK} 天，共 {} 個每日決策（布局／議題攻防／決戰三階段）；起始預算：{}　團隊信任：{:.1}/100　體力：{:.1}/100", CAMPAIGN_ROUNDS as u16 * DAYS_PER_WEEK as u16, game.budget, game.trust, game.stamina);
    println!(
        "每週會抽取一項陣營任務；達成可獲得最多 +1 預算。目標是遊戲任務，不代表現實政治建議。\n"
    );
    println!(
        "選民結構：{} 個遊戲用選民原型；權重是可調模型參數，不代表實際人口或選民占比。",
        VOTER_SEGMENTS.len()
    );
    println!("指標口徑：候選人支持度、政府施政滿意度、議題評價／關注度與團隊信任分開建模；都是情境值，不是實際民調。\n");
    println!(
        "玩家能力值（遊戲建構，非歷史評分）：政策 {:.0}／組織 {:.0}／溝通 {:.0}\n",
        game.player_attributes.policy,
        game.player_attributes.organization,
        game.player_attributes.communication
    );

    if !game.events.is_empty() {
        println!(
            "本情境公共議題（事件本身不預設政黨得失；回應方式才可能改變遊戲內候選人支持度）："
        );
        for event in &game.events {
            println!("  • {}：{}", event.year, event.title);
        }
        println!();
    }
    print_poll_summary(game);
    println!("口徑提醒：候選人支持度、政府施政滿意度、議題評價與幕僚團隊信任分開建模；均為情境值，不是實際民調。\n");
    println!("行動對候選人支持度的效果會受團隊信任、議題契合、固定 seed 的小幅波動，以及對手回應影響。\n");
}

fn print_player_assignment(game: &GameState) {
    let player = &game.candidates[game.player_index];
    let context = if game.region.is_some() {
        "地方政黨情境代表"
    } else if game.campaign_year == game.baseline_year {
        "年度虛構選戰候選人"
    } else {
        "全國政黨情境代表"
    };
    println!("\n【本場玩家身分】");
    println!("扮演角色：{}", game.side.label());
    println!(
        "玩家操控：{}｜政黨／名單：{}（{}）",
        player.name, player.party, context
    );
    let opponents = game
        .candidates
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != game.player_index)
        .map(|(_, candidate)| format!("{}（{}）", candidate.name, candidate.party))
        .collect::<Vec<_>>()
        .join("、");
    println!("其他競爭者：{opponents}");
}

fn print_poll_table(game: &GameState) {
    let player_name = &game.candidates[game.player_index].name;
    println!(
        "完整支持度檢視｜候選人選舉支持度、政黨票支持度、分眾與互比情境（遊戲模型值，非實際民調）"
    );
    for (index, candidate) in game.candidates.iter().enumerate() {
        let marker = if index == game.player_index {
            "▶"
        } else {
            " "
        };
        let bar = "█".repeat((game.polls[index] / 2.0).round() as usize);
        println!(
            "{marker} {:<22} {:>5.2}%（較起始盤 {:+.2}） {bar}",
            candidate.name,
            game.polls[index],
            game.polls[index] - candidate.starting_share
        );
    }
    print_party_support_table(game);
    println!("分眾支持度（原型權重／{player_name} 支持度）— 僅供觀察遊戲模型：");
    for segment in &game.voter_segments {
        let player_support = segment.polls.get(game.player_index).copied().unwrap_or(0.0);
        println!(
            "  {:<20} {:>4.0}%／{:>5.1}%",
            segment.name, segment.weight_pct, player_support
        );
    }
    let opinion = &game.government_opinion;
    let salient_issue = opinion.most_salient_issue();
    let issue_pressures = opinion.ranked_issue_signals();
    println!(
        "政府施政滿意度｜整體 {:.1}/100；最受關注議題：{}（議題評價 {:.1}/100、關注度 {:.1}/100；遊戲模型值）",
        opinion.administration_approval,
        issue_label(salient_issue),
        opinion.issue_rating(salient_issue),
        opinion.issue_priority(salient_issue)
    );
    println!("議題處理優先序（關注度 × 評價落差；僅為策略提示）：");
    for signal in issue_pressures.iter().take(3) {
        println!(
            "  {}｜評價 {:.1}、關注 {:.1}、待改善壓力 {:.1}",
            issue_label(signal.issue),
            signal.rating,
            signal.attention,
            signal.pressure
        );
    }
    let role_hint = match game.side {
        Side::Incumbent => "執政方可檢視高壓議題的施政缺口；政策回應不會直接等同候選人支持度變化。",
        Side::Challenger => {
            "在野方可評估高壓議題的監督與替代方案；議題評價不會直接等同候選人支持度變化。"
        }
    };
    println!("{role_hint}");
    println!("政府評價、候選人選舉支持度與團隊信任分開計算；此面板不是民調。\n");
    if game.candidates.len() > 2 {
        println!("互比式簡化情境：把其他候選人的支持平均分配給比較雙方；這不是實際互比民調。");
        for (index, candidate) in game
            .candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != game.player_index)
        {
            let other_candidates =
                (100.0 - game.polls[game.player_index] - game.polls[index]).max(0.0);
            let player_share = game.polls[game.player_index] + other_candidates * 0.5;
            println!(
                "  若比較「{}」與「{}」：{:.1}% 對 {:.1}%",
                game.candidates[game.player_index].name,
                candidate.name,
                player_share,
                100.0 - player_share
            );
        }
    }
}

fn print_poll_summary(game: &GameState) {
    println!("候選人選舉支持度（對比式遊戲情境，非實際民調）｜本場快照");
    for (index, candidate) in game.candidates.iter().enumerate() {
        let marker = if index == game.player_index {
            "▶"
        } else {
            " "
        };
        println!(
            "{marker} {}：{:.2}%（較起始盤 {:+.2} 點）",
            candidate.name,
            game.polls[index],
            game.polls[index] - candidate.starting_share
        );
    }
    print_party_support_table(game);
    let opinion = &game.government_opinion;
    let salient_issue = opinion.most_salient_issue();
    let pressure_issue = opinion.highest_pressure_issue();
    println!(
        "政府施政滿意度情境值：{:.1}/100；最受關注：{}（評價 {:.1}/100）；優先檢視：{}（待改善壓力 {:.1}/100）。此數值不等於候選人支持度。",
        opinion.administration_approval,
        issue_label(salient_issue),
        opinion.issue_rating(salient_issue),
        issue_label(pressure_issue.issue),
        pressure_issue.pressure
    );
    if game.candidates.len() > 2 {
        println!("互比式情境為遊戲簡化試算；完整分眾與各組互比請在行動選單輸入 p。每位候選人比例合計 100%。");
        for (index, candidate) in game
            .candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != game.player_index)
        {
            let other_candidates =
                (100.0 - game.polls[game.player_index] - game.polls[index]).max(0.0);
            let player_share = game.polls[game.player_index] + other_candidates * 0.5;
            println!(
                "  若比較「{}」與「{}」：{:.1}% 對 {:.1}%",
                game.candidates[game.player_index].name,
                candidate.name,
                player_share,
                100.0 - player_share
            );
        }
    }
}

fn current_party_support(game: &GameState) -> Result<Vec<PartyTicket>, String> {
    let candidates = game
        .candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            (
                candidate.party.to_string(),
                candidate.name.clone(),
                candidate.starting_share,
                game.polls[index],
            )
        })
        .collect::<Vec<_>>();
    party_ticket_support(
        &candidates,
        game.candidates[game.player_index].party,
        game.trust,
        game.turnout_momentum,
    )
    .map_err(str::to_string)
}

fn print_party_support_table(game: &GameState) {
    if game.region.is_some() {
        println!(
            "政黨票支持度：地方候選人選舉情境未另設政黨票；目前操控政黨為 {}。",
            game.candidates[game.player_index].party
        );
        return;
    }
    println!("政黨票支持度（全國不分區遊戲情境，非實際民調）：");
    match current_party_support(game) {
        Ok(tickets) => {
            for ticket in tickets {
                let marker = if ticket.party == game.candidates[game.player_index].party {
                    "▶ 玩家政黨"
                } else {
                    "　"
                };
                println!("  {marker}｜{}：{:.2}%", ticket.party, ticket.support_pct);
            }
        }
        Err(error) => println!("  暫時無法計算政黨票情境：{error}"),
    }
}

fn print_daily_poll_status(game: &GameState) {
    let player = &game.candidates[game.player_index];
    let player_share = game.polls[game.player_index];
    let rival = game
        .polls
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != game.player_index)
        .max_by(|(_, left), (_, right)| left.total_cmp(right));
    print!(
        "【即時局勢】候選人支持度（對比式）：▶ {} {:.2}%",
        player.name, player_share
    );
    if let Some((index, share)) = rival {
        print!(
            "｜目前最高競爭者 {} {:.2}%（差 {:+.2} 點）",
            game.candidates[index].name,
            share,
            player_share - share
        );
    }
    println!();
    if game.region.is_some() {
        println!(
            "             政黨狀態：地方候選人制，未另設政黨票支持度｜玩家政黨 {}",
            player.party
        );
    } else {
        match current_party_support(game) {
            Ok(tickets) => {
                let player_party_share = tickets
                    .iter()
                    .find(|ticket| ticket.party == player.party)
                    .map(|ticket| ticket.support_pct)
                    .unwrap_or(0.0);
                let leader = tickets
                    .iter()
                    .max_by(|left, right| left.support_pct.total_cmp(&right.support_pct));
                if let Some(leader) = leader {
                    println!(
                        "             政黨票支持度（不分區情境）：▶ {} {:.2}%｜領先政黨 {} {:.2}%",
                        player.party, player_party_share, leader.party, leader.support_pct
                    );
                }
            }
            Err(error) => println!("             政黨票支持度情境暫不可用：{error}"),
        }
    }
    println!("             以上皆為遊戲內支持度狀態，不是實際民調或選舉預測。");
}

fn run_interactive(
    game: &mut GameState,
    strategy: StrategyProfile,
    staffer_mode: bool,
    journal: &mut Vec<SaveCommand>,
    save_path: Option<&str>,
    save_config: &CampaignSaveConfig,
) -> Result<(), String> {
    'campaign: while !game.is_finished() {
        println!(
            "\n--- 第 {} / {} 週・第 {} / {} 天｜{} ---",
            game.round + 1,
            CAMPAIGN_ROUNDS,
            game.day + 1,
            DAYS_PER_WEEK,
            game.phase_label()
        );
        println!(
            "剩餘預算：{}　團隊信任：{:.1}　動員：{:.1}　體力：{:.1}",
            game.budget, game.trust, game.turnout_momentum, game.stamina
        );
        println!(
            "競選組織：總部 {}｜後援會 {}｜地面調度量能 {:.0}/55｜查核揭露 {} 次、累計懲處資源 {}",
            if game.headquarters_established {
                "已成立"
            } else {
                "未成立"
            },
            if game.supporters_association_established {
                "已成立"
            } else {
                "未成立"
            },
            game.field_capacity,
            game.exposed_tactics,
            game.sanctioned_funds
        );
        print_daily_poll_status(game);
        if let Some(event) = game.current_event() {
            println!("議題焦點：{} — {}", event.title, event.description);
        }
        print_campaign_daily_scene(game, staffer_mode);
        if let Some(objective) = game.current_objective() {
            let progress = game.current_objective_progress();
            let target = objective.target_actions;
            let bar = "■".repeat(progress as usize)
                + &"□".repeat(target.saturating_sub(progress) as usize);
            println!(
                "本週任務：{} [{}] {}/{} — {}",
                objective.title, bar, progress, target, objective.description
            );
        }
        if staffer_mode
            && game.day == 0
            && game.current_staffer_brief().is_some()
            && !run_interactive_staffer_brief(game, journal, save_path, save_config)?
        {
            break 'campaign;
        }
        if !staffer_mode {
            if let Some(beat) = game.current_story_beat() {
                println!("\n【多週情境｜第 {} 週】{}", beat.week, beat.title);
                println!("登場人物：{}", beat.character);
                println!("{}", beat.scene);
                println!(
                    "（人物與情境為虛構合成素材；選擇會改變遊戲資源與支持度，不耗用每日行動。）"
                );
                loop {
                    for (index, choice) in beat.choices.iter().enumerate() {
                        println!(
                        "  {}) {}｜預算 {}｜支持度 {:+.2}｜團隊信任 {:+.1}｜動員 {:+.1}｜體力 {:+.1}{}",
                        index + 1,
                        choice.label,
                        choice.budget_cost,
                        choice.support_delta,
                        choice.trust_delta,
                        choice.mobilization_delta,
                        choice.stamina_delta,
                        if choice.budget_cost > game.budget {
                            "（預算不足）"
                        } else {
                            ""
                        }
                    );
                        println!("     {}", choice.description);
                    }
                    let Some(selection) = select_story_choice(beat.choices.len())? else {
                        break 'campaign;
                    };
                    if selection == 0 {
                        game.skip_story_beat().map_err(|error| error.to_string())?;
                        journal.push(SaveCommand::SkipStory);
                        checkpoint_campaign(game, save_path, save_config, journal)?;
                        println!("本段情境暫不處理；後續故事仍會依目前進度展開。\n");
                        break;
                    }
                    match game.apply_story_choice(selection - 1) {
                        Ok(outcome) => {
                            journal.push(SaveCommand::StoryChoice(selection - 1));
                            checkpoint_campaign(game, save_path, save_config, journal)?;
                            print_story_outcome(game, &outcome);
                            break;
                        }
                        Err(error @ GameError::InsufficientBudget { .. }) => {
                            println!("{error}\n請改選其他解法或暫不處理。\n");
                        }
                        Err(error) => {
                            println!("{error}\n");
                            break;
                        }
                    }
                }
            }
        }
        let recommendations = strategy_engine_for(game, strategy, staffer_mode).rank_actions(game);
        let selected = select_interactive_action(game, &recommendations)?;
        let Some(action_id) = selected else {
            break;
        };
        match game.apply_action(action_id) {
            Ok(outcome) => {
                journal.push(SaveCommand::Action(action_id.index()));
                checkpoint_campaign(game, save_path, save_config, journal)?;
                print_action_outcome(game, &outcome, staffer_mode)
            }
            Err(message) => println!("{message}\n"),
        }
    }
    if !game.is_finished() && !game.is_settled() {
        journal.push(SaveCommand::SettleEarly);
        game.finish();
        checkpoint_campaign(game, save_path, save_config, journal)?;
    }
    Ok(())
}

fn select_story_choice(choice_count: usize) -> Result<Option<usize>, String> {
    loop {
        print!("選擇方案（1–{choice_count}，0 略過，? 說明，q 結束本場）：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        let value = input.trim();
        if value.eq_ignore_ascii_case("q") {
            if confirm_end_campaign()? {
                return Ok(None);
            }
            continue;
        }
        if value == "?" {
            println!("輸入編號採用方案，0 略過本段且不耗每日行動；選擇會改變資源與後續劇情。q 會詢問是否提前結算本場。\n");
            continue;
        }
        if let Ok(selection) = value.parse::<usize>() {
            if selection <= choice_count {
                return Ok(Some(selection));
            }
        }
        println!("請輸入有效的選項、0 或 q。\n");
    }
}

fn confirm_end_campaign() -> Result<bool, String> {
    print!("結束本場並立即結算？輸入 y 確認，其餘輸入返回：");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;
    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn print_action_menu_help() {
    println!(
        "\n行動方式：先選一個分類，再選行動；輸入 r 可執行策略引擎首選，p 可隨時查看支持度，? 可重看說明。分類內輸入 d 編號可先看行動描述與效果，數字則會直接執行；執行結果以 ✓【已選】標示，b 返回上一層。q 會先詢問是否提前結算。\n每日狀態分別列出候選人支持度與全國政黨票支持度；支持度預估是目前遊戲模型的一步期望，不是實際民調。每次每日行動都會推進一天，每週末附戰情回顧。\n"
    );
}

fn print_action_details(recommendation: &StrategyRecommendation) {
    let preview = &recommendation.preview;
    println!("\n【行動詳情｜{}】", preview.action_name);
    println!("{}", preview.action_description);
    println!(
        "預估影響：預算 -{}｜候選人支持度 {:+.2}｜團隊信任 {:+.2}｜動員 {:+.2}｜體力 {:+.1}",
        preview.cost,
        preview.expected_support_delta,
        preview.trust_delta,
        preview.turnout_delta,
        preview.stamina_delta
    );
    if preview.detection_probability > 0.0 {
        println!(
            "誠信風險：模型預估揭露率 {:.0}%，期望懲處資源 {:.1}。",
            preview.detection_probability * 100.0,
            preview.expected_sanction_fine
        );
    }
    if preview.infrastructure_delta > 0.0 {
        println!(
            "組織效果：地面調度量能 +{:.0}。",
            preview.infrastructure_delta
        );
    }
    let mut context = Vec::new();
    if preview.event_match {
        context.push("呼應本週公共議題");
    }
    if preview.local_match {
        context.push("符合地方關注");
    }
    if preview.objective_match {
        context.push("符合本週任務");
    }
    if !context.is_empty() {
        println!("情境契合：{}。", context.join("、"));
    }
    if !recommendation.rationales.is_empty() {
        println!("推薦理由：{}。", recommendation.rationales.join("；"));
    }
    println!(
        "策略分（排序用）{:.2}；支持度變化仍會受每日波動與對手回應影響。",
        recommendation.score
    );
}

fn run_interactive_staffer_brief(
    game: &mut GameState,
    journal: &mut Vec<SaveCommand>,
    save_path: Option<&str>,
    save_config: &CampaignSaveConfig,
) -> Result<bool, String> {
    let Some(brief) = game.current_staffer_brief() else {
        return Ok(true);
    };
    println!("\n【幕僚會議｜第 {} 週】{}", brief.week, brief.title);
    println!("登場人物：{}", brief.character);
    println!("{}", brief.scene);
    if let Some(event) = game.current_event() {
        println!("背景事件：{} — {}", event.title, event.description);
    } else {
        println!("背景事件：本週未排入事件卡；依照戰役局勢處理本次議程。");
    }
    if brief.index == 4 {
        print_staffer_poll_brief(game);
    }
    println!("方案可呼應本週事件或延續既定主軸，獲得小幅遊戲內加成；所有支持度都是情境模型值，不是實際民調。");
    loop {
        for (index, choice) in brief.choices.iter().enumerate() {
            let event_fit = game
                .current_event()
                .is_some_and(|event| event.issue == choice.issue);
            let theme_fit = game
                .staffer_progress
                .focus()
                .is_some_and(|focus| choice.affinity == Some(focus));
            let mut tags = Vec::new();
            if event_fit {
                tags.push("呼應本週事件");
            }
            if theme_fit {
                tags.push("延續競選主軸");
            }
            let tag = if tags.is_empty() {
                String::new()
            } else {
                format!("｜{}", tags.join("、"))
            };
            println!(
                "  {}) {}｜預算 {}｜支持度 {:+.2}｜團隊信任 {:+.1}｜動員 {:+.1}｜體力 {:+.1}{}{}",
                index + 1,
                choice.label,
                choice.budget_cost,
                choice.support_delta,
                choice.trust_delta,
                choice.mobilization_delta,
                choice.stamina_delta,
                tag,
                if choice.budget_cost > game.budget {
                    "（預算不足）"
                } else {
                    ""
                }
            );
            println!("     {}", choice.description);
        }
        let Some(selection) = select_story_choice(brief.choices.len())? else {
            return Ok(false);
        };
        if selection == 0 {
            game.skip_staffer_brief()
                .map_err(|error| error.to_string())?;
            journal.push(SaveCommand::SkipStaffer);
            checkpoint_campaign(game, save_path, save_config, journal)?;
            println!("關係與承諾更新：{}", game.staffer_progress.memory_summary());
            println!("本次幕僚會議暫不處理；戰役每日行動照常繼續。\n");
            return Ok(true);
        }
        match game.apply_staffer_choice(selection - 1) {
            Ok(outcome) => {
                journal.push(SaveCommand::StafferChoice(selection - 1));
                checkpoint_campaign(game, save_path, save_config, journal)?;
                print_staffer_outcome(game, &outcome);
                return Ok(true);
            }
            Err(error @ GameError::InsufficientBudget { .. }) => {
                println!("{error}\n請改選其他方案或略過本次會議。\n");
            }
            Err(error) => {
                println!("{error}\n");
                return Ok(true);
            }
        }
    }
}

fn print_staffer_poll_brief(game: &GameState) {
    let player = &game.candidates[game.player_index];
    let player_share = game.polls[game.player_index];
    let rival = game
        .candidates
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != game.player_index)
        .max_by(|(left_index, _), (right_index, _)| {
            game.polls[*left_index].total_cmp(&game.polls[*right_index])
        });
    println!("候選人支持度檢視（遊戲模型值，非實際民調）：");
    println!(
        "  {}：{:.2}%（相對起始盤 {:+.2} 點）",
        player.name,
        player_share,
        player_share - player.starting_share
    );
    if let Some((index, candidate)) = rival {
        println!(
            "  目前支持度最高的競爭者 {}：{:.2}%",
            candidate.name, game.polls[index]
        );
    }
}

fn print_campaign_daily_scene(game: &GameState, staffer_mode: bool) {
    let week = game.round + 1;
    let day = game.day + 1;
    if let Some((arc, scene)) = staffer_daily_scene(week, day) {
        if staffer_mode {
            println!("\n【今日選戰｜第 {week} 週・第 {day} 天｜{arc}】{scene}");
        } else {
            let candidate_name = &game.candidates[game.player_index].name;
            println!("\n【候選人行程｜{candidate_name}｜第 {week} 週・第 {day} 天｜{arc}】{scene}");
        }
    }
    let route_echo = if staffer_mode {
        staffer_daily_route_echo(week, day, game.staffer_progress)
    } else {
        candidate_daily_route_echo(week, day, game.story_progress)
    };
    if let Some(echo) = route_echo {
        println!("劇情回聲：{echo}");
    }
    if staffer_mode {
        if let Some(echo) = staffer_character_memory_echo(week, day, game.staffer_progress) {
            println!("關係回訪：{echo}");
        }
    }
    if !staffer_mode && game.day == 0 {
        if let Some(echo) = project_twn::story::character_memory_echo(week, game.story_progress) {
            println!("人物記憶回訪：{echo}");
        }
    }
    if staffer_mode && game.day == 0 {
        let reminder = match game.side {
            Side::Incumbent => "執政方幕僚備忘：清楚區分已完成、進行中與超出權責的承諾。",
            Side::Challenger => "在野方幕僚備忘：每項監督或批評都要接上替代方案、時程與資源來源。",
        };
        println!("{reminder}");
        let opinion = &game.government_opinion;
        let issue = opinion.most_salient_issue();
        let priority = opinion.highest_pressure_issue();
        println!(
            "幕僚議題簡報：施政滿意度 {:.1}/100；最受關注「{}」，評價 {:.1}/100；優先檢視「{}」，待改善壓力 {:.1}/100。候選人支持度、政府評價與團隊信任分開；這是遊戲情境值，不是民調。",
            opinion.administration_approval,
            issue_label(issue),
            opinion.issue_rating(issue),
            issue_label(priority.issue),
            priority.pressure
        );
    }
    if staffer_mode && (day == 1 || day == 7) {
        if let Some(focus) = game.staffer_progress.focus() {
            println!(
                "幕僚主軸回扣：{}；若今天改變訊息，記得向團隊交代原因。",
                focus.label()
            );
        }
    }
}

fn print_staffer_outcome(game: &GameState, outcome: &StafferOutcome) {
    let candidate_name = &game.candidates[game.player_index].name;
    println!(
        "✓【幕僚已選】{}：{} 支持度 {:.2}% → {:.2}%（{:+.2} 點），團隊信任 {:.1}、動員 {:.1}、體力 {:.1}、預算 {}",
        outcome.title,
        candidate_name,
        outcome.support_before,
        outcome.support_after,
        outcome.support_delta,
        outcome.trust_after,
        outcome.mobilization_after,
        outcome.stamina_after,
        outcome.budget_after
    );
    if let Some(note) = outcome.note {
        println!("  {note}");
    }
    println!(
        "  關係與承諾更新：{}",
        game.staffer_progress.memory_summary()
    );
}

fn print_staffer_summary(game: &GameState) {
    println!(
        "幕僚模式回顧：完成／略過 {} 次幕僚會議；競選主軸：{}；決策路線：{}",
        game.staffer_progress.completed_briefings(),
        game.staffer_progress
            .focus()
            .map(|focus| focus.label())
            .unwrap_or("未設定"),
        game.staffer_progress.summary()
    );
    println!("{}", game.staffer_progress.memory_summary());
    println!("{}", game.staffer_progress.ending_note());
}

fn print_candidate_story_summary(game: &GameState) {
    println!(
        "候選人劇情回顧：完成 {}／6 段；決策路線：{}",
        game.story_progress.completed_beats(),
        game.story_progress.summary()
    );
    println!("{}", game.story_progress.memory_summary());
    println!("{}", candidate_story_ending_note(game.story_progress));
}

fn print_story_outcome(game: &GameState, outcome: &StoryOutcome) {
    let candidate_name = &game.candidates[game.player_index].name;
    println!(
        "✓【故事已選】{}｜{}：{} 支持度 {:.2}% → {:.2}%（{:+.2} 點），團隊信任 {:.1}、動員 {:.1}、體力 {:.1}、預算 {}",
        outcome.title,
        outcome.choice_label,
        candidate_name,
        outcome.support_before,
        outcome.support_after,
        outcome.support_delta,
        outcome.trust_after,
        outcome.mobilization_after,
        outcome.stamina_after,
        outcome.budget_after
    );
    println!("人物記憶更新：{}", game.story_progress.memory_summary());
    if let Some(note) = outcome.note {
        println!("  {note}");
    }
}

fn strategy_engine_for(
    game: &GameState,
    strategy: StrategyProfile,
    staffer_mode: bool,
) -> StrategyEngine {
    let engine = StrategyEngine::new(strategy);
    if staffer_mode {
        if let Some(focus) = game.staffer_progress.focus() {
            return engine.with_campaign_focus(focus);
        }
    }
    engine
}

fn select_interactive_action(
    game: &GameState,
    recommendations: &[StrategyRecommendation],
) -> Result<Option<ActionId>, String> {
    loop {
        let categories: Vec<_> = ActionMenuCategory::ALL
            .iter()
            .copied()
            .filter_map(|category| {
                let options: Vec<_> = recommendations
                    .iter()
                    .filter(|recommendation| category.includes(recommendation.preview.id))
                    .collect();
                (!options.is_empty()).then_some((category, options))
            })
            .collect();

        println!("\n【今日行動｜先選分類】依目前陣營與預算篩選：");
        for (index, (category, options)) in categories.iter().enumerate() {
            let top = &options[0];
            println!(
                "  {}) {}（{} 項；本類推薦：{}，預估支持 {:+.2}）",
                index + 1,
                category.label(),
                options.len(),
                top.preview.action_name,
                top.preview.expected_support_delta
            );
        }
        println!("  r) 執行策略推薦　p) 查看支持度　?) 操作說明　q) 結束本場");
        print!("選分類：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        let choice = input.trim();
        if choice.eq_ignore_ascii_case("q") {
            if confirm_end_campaign()? {
                return Ok(None);
            }
            continue;
        }
        if choice.eq_ignore_ascii_case("r") {
            return Ok(recommendations
                .first()
                .map(|recommendation| recommendation.preview.id));
        }
        if choice.eq_ignore_ascii_case("p") {
            print_poll_table(game);
            continue;
        }
        if choice == "?" {
            print_action_menu_help();
            continue;
        }
        let Ok(index) = choice.parse::<usize>() else {
            println!("請輸入分類編號、r、p、? 或 q。\n");
            continue;
        };
        let Some((category, options)) = categories.get(index.saturating_sub(1)) else {
            println!("沒有這個分類。\n");
            continue;
        };

        loop {
            println!("\n【{}｜先看排序，再選要執行的行動】", category.label());
            for (option_index, recommendation) in options.iter().enumerate() {
                let risk = if recommendation.preview.detection_probability > 0.0 {
                    format!(
                        "｜預估揭露 {:.0}%",
                        recommendation.preview.detection_probability * 100.0
                    )
                } else {
                    String::new()
                };
                let infrastructure = if recommendation.preview.infrastructure_delta > 0.0 {
                    format!(
                        "｜組織量能 +{:.0}",
                        recommendation.preview.infrastructure_delta
                    )
                } else {
                    String::new()
                };
                println!(
                    "  {}) {}{} [{}]｜預算 -{}｜支持度 {:+.2}｜體力 {:+.1}{}{}",
                    option_index + 1,
                    if option_index == 0 {
                        "★ 推薦｜"
                    } else {
                        ""
                    },
                    recommendation.preview.action_name,
                    recommendation.preview.category.label(),
                    recommendation.preview.cost,
                    recommendation.preview.expected_support_delta,
                    recommendation.preview.stamina_delta,
                    risk,
                    infrastructure
                );
            }
            print!("選行動（d 編號看詳情，r 本類推薦，p 支持度，b 返回，? 說明，q 結束）：");
            io::stdout().flush().map_err(|error| error.to_string())?;
            input.clear();
            io::stdin()
                .read_line(&mut input)
                .map_err(|error| error.to_string())?;
            let selected = input.trim();
            if selected.eq_ignore_ascii_case("b") {
                break;
            }
            if selected.eq_ignore_ascii_case("q") {
                if confirm_end_campaign()? {
                    return Ok(None);
                }
                continue;
            }
            if selected.eq_ignore_ascii_case("r") {
                return Ok(options
                    .first()
                    .map(|recommendation| recommendation.preview.id));
            }
            if selected.eq_ignore_ascii_case("p") {
                print_poll_table(game);
                continue;
            }
            if selected == "?" {
                print_action_menu_help();
                continue;
            }
            if let Some(number) = selected
                .strip_prefix('d')
                .or_else(|| selected.strip_prefix('D'))
            {
                let Ok(option_index) = number.trim().parse::<usize>() else {
                    println!("請輸入 d 加行動編號，例如 d 2。\n");
                    continue;
                };
                if let Some(recommendation) = options.get(option_index.saturating_sub(1)) {
                    print_action_details(recommendation);
                } else {
                    println!("沒有這個行動。\n");
                }
                continue;
            }
            let Ok(option_index) = selected.parse::<usize>() else {
                println!("請輸入行動編號、d 編號、r、p、b、? 或 q。\n");
                continue;
            };
            if let Some(recommendation) = options.get(option_index.saturating_sub(1)) {
                return Ok(Some(recommendation.preview.id));
            }
            println!("沒有這個行動。\n");
        }
    }
}

#[derive(Clone, Copy, Default)]
struct AutoRunSummary {
    actions_taken: u16,
    action_counts: [u16; 3],
    action_usage_counts: [u16; ActionId::COUNT],
    module_contribution_totals: [f64; StrategyAxis::COUNT],
}

impl AutoRunSummary {
    fn record(&mut self, recommendation: &StrategyRecommendation) {
        self.actions_taken += 1;
        let category_index = match recommendation.preview.category {
            TacticCategory::Positive => 0,
            TacticCategory::Neutral => 1,
            TacticCategory::Negative => 2,
        };
        self.action_counts[category_index] += 1;
        self.action_usage_counts[recommendation.preview.id.index()] += 1;
        for (total, contribution) in self
            .module_contribution_totals
            .iter_mut()
            .zip(recommendation.module_contributions)
        {
            *total += contribution;
        }
    }

    fn mean_module_contributions(self) -> [f64; StrategyAxis::COUNT] {
        if self.actions_taken == 0 {
            return [0.0; StrategyAxis::COUNT];
        }
        self.module_contribution_totals
            .map(|total| total / f64::from(self.actions_taken))
    }
}

fn run_auto(
    game: &mut GameState,
    show_progress: bool,
    strategy: StrategyProfile,
) -> AutoRunSummary {
    run_auto_mode(game, show_progress, strategy, false)
}

fn run_auto_staffer(
    game: &mut GameState,
    show_progress: bool,
    strategy: StrategyProfile,
) -> AutoRunSummary {
    run_auto_mode(game, show_progress, strategy, true)
}

fn run_auto_mode(
    game: &mut GameState,
    show_progress: bool,
    strategy: StrategyProfile,
    staffer_mode: bool,
) -> AutoRunSummary {
    let mut summary = AutoRunSummary::default();
    while !game.is_finished() {
        if show_progress {
            print_campaign_daily_scene(game, staffer_mode);
        }
        if staffer_mode {
            if let Some(brief) = game.current_staffer_brief() {
                if show_progress {
                    println!("\n【幕僚會議｜第 {} 週】{}", brief.week, brief.title);
                    println!("{}", brief.scene);
                    if let Some(event) = game.current_event() {
                        println!("背景事件：{} — {}", event.title, event.description);
                    }
                    if brief.index == 4 {
                        print_staffer_poll_brief(game);
                    }
                }
                let selected = brief
                    .choices
                    .iter()
                    .enumerate()
                    .filter(|(_, choice)| choice.budget_cost <= game.budget)
                    .max_by(|(_, left), (_, right)| {
                        let score = |choice: &project_twn::staffer::StafferChoiceSpec| {
                            let continuity = staffer_continuity_bonus(
                                brief.index,
                                choice,
                                game.staffer_progress,
                            );
                            let event_match = game
                                .current_event()
                                .is_some_and(|event| event.issue == choice.issue);
                            choice.support_delta * 3.0
                                + (choice.trust_delta + continuity.trust_delta) * 0.08
                                + choice.mobilization_delta * 0.05
                                + choice.stamina_delta * 0.02
                                - f64::from(choice.budget_cost) * 0.12
                                + continuity.support_delta * 3.0
                                + if event_match { 0.12 } else { 0.0 }
                        };
                        score(left).total_cmp(&score(right))
                    })
                    .map(|(index, _)| index);
                if let Some(index) = selected {
                    if let Ok(outcome) = game.apply_staffer_choice(index) {
                        if show_progress {
                            print_staffer_outcome(game, &outcome);
                        }
                    }
                } else {
                    let _ = game.skip_staffer_brief();
                    if show_progress {
                        println!("關係與承諾更新：{}", game.staffer_progress.memory_summary());
                        println!("本次幕僚會議因預算不足而略過。\n");
                    }
                }
            }
        }
        if !staffer_mode {
            if let Some(beat) = game.current_story_beat() {
                if show_progress {
                    println!("\n【多週情境｜第 {} 週】{}", beat.week, beat.title);
                    println!("登場人物：{}", beat.character);
                    println!("{}", beat.scene);
                    println!("（情境與人物為虛構合成素材；選擇會影響遊戲資源與後續敘事。）");
                }
                let selected = beat
                    .choices
                    .iter()
                    .enumerate()
                    .filter(|(_, choice)| choice.budget_cost <= game.budget)
                    .max_by(|(_, left), (_, right)| {
                        let score = |choice: &project_twn::story::StoryChoiceSpec| {
                            let bonus = coherence_bonus(beat.index, choice.id, game.story_progress);
                            choice.support_delta * 3.0
                                + (choice.trust_delta + bonus.trust_delta) * 0.08
                                + choice.mobilization_delta * 0.05
                                + choice.stamina_delta * 0.02
                                - f64::from(choice.budget_cost) * 0.12
                                + bonus.support_delta * 3.0
                        };
                        score(left).total_cmp(&score(right))
                    })
                    .map(|(index, _)| index);
                if let Some(index) = selected {
                    if let Ok(outcome) = game.apply_story_choice(index) {
                        if show_progress {
                            print_story_outcome(game, &outcome);
                        }
                    }
                } else {
                    let _ = game.skip_story_beat();
                    if show_progress {
                        println!("本段情境因預算不足而暫不處理；後續劇情仍會依目前進度展開。\n");
                    }
                }
            }
        }
        let Some(recommendation) = strategy_engine_for(game, strategy, staffer_mode)
            .rank_actions(game)
            .into_iter()
            .next()
        else {
            break;
        };
        let Ok(outcome) = game.apply_action(recommendation.preview.id) else {
            break;
        };
        summary.record(&recommendation);
        if show_progress {
            print_action_outcome(game, &outcome, staffer_mode);
        }
    }
    summary
}

#[derive(Clone, Debug)]
struct BatchRecord {
    style: CampaignStyle,
    strategy: StrategyProfile,
    seed: u64,
    sensitivity_parameter: &'static str,
    multiplier: f64,
    sensitivity_parameter_2: &'static str,
    multiplier_2: f64,
    winner_candidate: String,
    winner_party: String,
    player_party: String,
    player_won: bool,
    player_win_probability: f64,
    player_final_share: f64,
    signed_margin: f64,
    objectives_completed: u8,
    budget_remaining: i32,
    stamina_remaining: f64,
    exposed_tactics: u8,
    sanctioned_funds: i32,
    action_counts: [u16; 3],
    action_usage_counts: [u16; ActionId::COUNT],
    module_contribution_means: [f64; StrategyAxis::COUNT],
    trust: f64,
    mobilization: f64,
    scorecard: CampaignScorecard,
    story_path: String,
}

#[derive(Clone, Copy)]
struct SensitivitySetting {
    parameter: &'static str,
    multiplier: f64,
    parameter_2: &'static str,
    multiplier_2: f64,
}

#[derive(Clone, Debug)]
struct BatchMetrics {
    seed_win_rate: f64,
    mean_model_probability: f64,
    mean_final_share: f64,
    mean_signed_margin: f64,
    mean_objectives: f64,
    mean_exposed_tactics: f64,
    mean_sanctioned_funds: f64,
    mean_budget_remaining: f64,
    mean_stamina_remaining: f64,
    mean_action_counts: [f64; 3],
    mean_module_contributions: [f64; StrategyAxis::COUNT],
    shares: Vec<f64>,
}

fn run_batch(
    options: &Options,
    runs: u32,
    role: Side,
    region: Option<&'static RegionProfile>,
    selected_party: Option<&'static PartyProfile>,
) -> Result<(), String> {
    if options.interaction_sensitivity {
        return run_interaction_sensitivity_batch(options, runs, role, region, selected_party);
    }
    if options.sensitivity {
        return run_sensitivity_batch(options, runs, role, region, selected_party);
    }
    let last_seed = options
        .seed
        .checked_add(u64::from(runs - 1))
        .ok_or_else(|| "seed 範圍超出 u64 上限。".to_string())?;
    let styles: Vec<_> = if options.sweep_styles {
        vec![
            CampaignStyle::Balanced,
            CampaignStyle::Policy,
            CampaignStyle::Organizer,
            CampaignStyle::Communicator,
        ]
    } else {
        vec![options.style]
    };
    let strategies = if options.sweep_strategies {
        StrategyProfile::ALL.to_vec()
    } else {
        vec![options.strategy]
    };
    let mut records = Vec::with_capacity(runs as usize * styles.len() * strategies.len());

    for &strategy in &strategies {
        for &style in &styles {
            let scenario = BatchScenario {
                options,
                role,
                style,
                region,
                selected_party,
            };
            for offset in 0..runs {
                let seed = options.seed + u64::from(offset);
                records.push(simulate_batch_record(
                    &scenario,
                    seed,
                    ModelParameters::default(),
                    SensitivitySetting {
                        parameter: "baseline",
                        multiplier: 1.0,
                        parameter_2: "none",
                        multiplier_2: 1.0,
                    },
                    strategy,
                ));
            }
        }
    }

    println!("=== 批次模擬摘要 ===");
    println!(
        "情境：{} 年、{}、{}；seed：{}–{}；每種能力風格／策略 {} 場。",
        options.year,
        role.label(),
        region.map(|value| value.name).unwrap_or("總統全國情境"),
        options.seed,
        last_seed,
        runs
    );
    println!(
        "難度：{}；事件包：{}；事件時序：{}。",
        options.setup.difficulty.label(),
        options.setup.event_pack.label(),
        options.setup.timeline.label()
    );
    if let Some(party) = selected_party {
        println!("玩家政黨情境：{}。", party.name);
    }
    for strategy in &strategies {
        for style in &styles {
            let group: Vec<_> = records
                .iter()
                .filter(|record| {
                    record.style == *style
                        && record.strategy == *strategy
                        && record.sensitivity_parameter == "baseline"
                })
                .collect();
            let metrics = batch_metrics(&group);
            print_batch_metrics(
                &format!("{}／{}", style.label(), strategy.label()),
                &metrics,
            );
        }
    }
    println!(
        "結果是固定策略模組下的遊戲模型跑次，不是民調或選舉預測；策略評分權重是透明的遊戲啟發式。"
    );

    if let Some(path) = options.batch_log.as_deref() {
        save_batch_log(path, options, role, region, &records)?;
        println!("逐場原始結果已追加至：{path}");
    }
    Ok(())
}

fn simulate_batch_record(
    scenario: &BatchScenario<'_>,
    seed: u64,
    model_parameters: ModelParameters,
    sensitivity: SensitivitySetting,
    strategy: StrategyProfile,
) -> BatchRecord {
    let mut game = new_scenario_game(scenario, seed, model_parameters);
    let auto_summary = run_auto(&mut game, false, strategy);
    let result = game.finish();
    let winner = &game.candidates[result.winner_index];
    let player = &game.candidates[game.player_index];
    let player_final_share = result.shares[game.player_index];
    let strongest_opponent_share = result
        .shares
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != game.player_index)
        .map(|(_, share)| *share)
        .fold(0.0, f64::max);
    BatchRecord {
        style: scenario.style,
        strategy,
        seed,
        sensitivity_parameter: sensitivity.parameter,
        multiplier: sensitivity.multiplier,
        sensitivity_parameter_2: sensitivity.parameter_2,
        multiplier_2: sensitivity.multiplier_2,
        winner_candidate: winner.name.clone(),
        winner_party: winner.party.to_string(),
        player_party: player.party.to_string(),
        player_won: result.winner_index == game.player_index,
        player_win_probability: result.player_win_probability,
        player_final_share,
        signed_margin: player_final_share - strongest_opponent_share,
        objectives_completed: game.objectives_completed(),
        budget_remaining: game.budget,
        stamina_remaining: game.stamina,
        exposed_tactics: game.exposed_tactics,
        sanctioned_funds: game.sanctioned_funds,
        action_counts: auto_summary.action_counts,
        action_usage_counts: auto_summary.action_usage_counts,
        module_contribution_means: auto_summary.mean_module_contributions(),
        trust: game.trust,
        mobilization: game.turnout_momentum,
        scorecard: game.scorecard(),
        story_path: game.story_progress.summary(),
    }
}

fn run_sensitivity_batch(
    options: &Options,
    runs: u32,
    role: Side,
    region: Option<&'static RegionProfile>,
    selected_party: Option<&'static PartyProfile>,
) -> Result<(), String> {
    let last_seed = options
        .seed
        .checked_add(u64::from(runs - 1))
        .ok_or_else(|| "seed 範圍超出 u64 上限。".to_string())?;
    let scenario = BatchScenario {
        options,
        role,
        style: options.style,
        region,
        selected_party,
    };
    let mut records = Vec::with_capacity(runs as usize * (1 + SensitivityParameter::ALL.len() * 2));
    for offset in 0..runs {
        let seed = options.seed + u64::from(offset);
        records.push(simulate_batch_record(
            &scenario,
            seed,
            ModelParameters::default(),
            SensitivitySetting {
                parameter: "baseline",
                multiplier: 1.0,
                parameter_2: "none",
                multiplier_2: 1.0,
            },
            options.strategy,
        ));
    }
    for parameter in SensitivityParameter::ALL {
        for multiplier in [0.8, 1.2] {
            let model_parameters = ModelParameters::default()
                .with_multiplier(parameter, multiplier)
                .map_err(str::to_string)?;
            for offset in 0..runs {
                let seed = options.seed + u64::from(offset);
                records.push(simulate_batch_record(
                    &scenario,
                    seed,
                    model_parameters,
                    SensitivitySetting {
                        parameter: parameter.key(),
                        multiplier,
                        parameter_2: "none",
                        multiplier_2: 1.0,
                    },
                    options.strategy,
                ));
            }
        }
    }

    println!("=== 模型參數單因子敏感度（±20%）===");
    println!(
        "情境：{} 年、{}、{}、能力風格 {}、策略 {}；配對 seed：{}–{}；每項比較 {} 場。",
        options.year,
        role.label(),
        region.map(|value| value.name).unwrap_or("總統全國情境"),
        options.style.label(),
        options.strategy.label(),
        options.seed,
        last_seed,
        runs
    );
    if let Some(party) = selected_party {
        println!("玩家政黨情境：{}。", party.name);
    }
    let baseline: Vec<_> = records
        .iter()
        .filter(|record| record.sensitivity_parameter == "baseline")
        .collect();
    let baseline_metrics = batch_metrics(&baseline);
    print_batch_metrics("基準參數", &baseline_metrics);
    let baseline_by_seed: HashMap<_, _> = baseline
        .iter()
        .map(|record| (record.seed, *record))
        .collect();
    for parameter in SensitivityParameter::ALL {
        for multiplier in [0.8, 1.2] {
            let group: Vec<_> = records
                .iter()
                .filter(|record| {
                    record.sensitivity_parameter == parameter.key()
                        && (record.multiplier - multiplier).abs() < f64::EPSILON
                })
                .collect();
            let metrics = batch_metrics(&group);
            let paired_share_delta = group
                .iter()
                .map(|record| {
                    record.player_final_share - baseline_by_seed[&record.seed].player_final_share
                })
                .sum::<f64>()
                / group.len() as f64;
            let paired_probability_delta = group
                .iter()
                .map(|record| {
                    record.player_win_probability
                        - baseline_by_seed[&record.seed].player_win_probability
                })
                .sum::<f64>()
                / group.len() as f64;
            println!(
                "{} × {:.1}：終局支持配對變化 {:+.2} 個百分點｜模型內勝率配對變化 {:+.2} 個百分點｜seed 勝出率 {:.1}%｜平均票差 {:+.2}",
                parameter.label(),
                multiplier,
                paired_share_delta,
                paired_probability_delta * 100.0,
                metrics.seed_win_rate * 100.0,
                metrics.mean_signed_margin
            );
        }
    }
    println!("僅單次調整一個遊戲參數；這是模型內方向性檢查，不代表參數已由選舉資料校準。");

    if let Some(path) = options.batch_log.as_deref() {
        save_batch_log(path, options, role, region, &records)?;
        println!("逐場原始結果已追加至：{path}");
    }
    Ok(())
}

fn run_interaction_sensitivity_batch(
    options: &Options,
    runs: u32,
    role: Side,
    region: Option<&'static RegionProfile>,
    selected_party: Option<&'static PartyProfile>,
) -> Result<(), String> {
    let last_seed = options
        .seed
        .checked_add(u64::from(runs - 1))
        .ok_or_else(|| "seed 範圍超出 u64 上限。".to_string())?;
    let scenario = BatchScenario {
        options,
        role,
        style: options.style,
        region,
        selected_party,
    };
    let parameter_pairs =
        SensitivityParameter::ALL.len() * (SensitivityParameter::ALL.len() - 1) / 2;
    let mut records = Vec::with_capacity(runs as usize * parameter_pairs * 4);

    for (first_index, first) in SensitivityParameter::ALL.iter().copied().enumerate() {
        for second in SensitivityParameter::ALL
            .iter()
            .copied()
            .skip(first_index + 1)
        {
            for (first_multiplier, second_multiplier) in
                [(0.8, 0.8), (0.8, 1.2), (1.2, 0.8), (1.2, 1.2)]
            {
                let model_parameters = ModelParameters::default()
                    .with_multiplier(first, first_multiplier)
                    .and_then(|parameters| parameters.with_multiplier(second, second_multiplier))
                    .map_err(str::to_string)?;
                for offset in 0..runs {
                    records.push(simulate_batch_record(
                        &scenario,
                        options.seed + u64::from(offset),
                        model_parameters,
                        SensitivitySetting {
                            parameter: first.key(),
                            multiplier: first_multiplier,
                            parameter_2: second.key(),
                            multiplier_2: second_multiplier,
                        },
                        options.strategy,
                    ));
                }
            }
        }
    }

    println!("=== 模型參數二因子交互作用敏感度（0.8×／1.2× 2×2 配對）===");
    println!(
        "情境：{} 年、{}、{}、能力風格 {}、策略 {}；配對 seed：{}–{}；每組 {} 場、共 {} 組參數交互作用。",
        options.year,
        role.label(),
        region.map(|value| value.name).unwrap_or("總統全國情境"),
        options.style.label(),
        options.strategy.label(),
        options.seed,
        last_seed,
        runs,
        parameter_pairs
    );
    if let Some(party) = selected_party {
        println!("玩家政黨情境：{}。", party.name);
    }

    for (first_index, first) in SensitivityParameter::ALL.iter().copied().enumerate() {
        for second in SensitivityParameter::ALL
            .iter()
            .copied()
            .skip(first_index + 1)
        {
            let corners_by_seed = interaction_corners(&records, first, second);
            let paired = corners_by_seed
                .values()
                .filter_map(|corners| Some([corners[0]?, corners[1]?, corners[2]?, corners[3]?]))
                .collect::<Vec<_>>();
            let count = paired.len() as f64;
            let share_contrast = paired
                .iter()
                .map(|corners| {
                    factorial_interaction(
                        corners[0].player_final_share,
                        corners[1].player_final_share,
                        corners[2].player_final_share,
                        corners[3].player_final_share,
                    )
                })
                .sum::<f64>()
                / count;
            let probability_contrast = paired
                .iter()
                .map(|corners| {
                    factorial_interaction(
                        corners[0].player_win_probability,
                        corners[1].player_win_probability,
                        corners[2].player_win_probability,
                        corners[3].player_win_probability,
                    )
                })
                .sum::<f64>()
                / count;
            println!(
                "{} × {}：終局支持 ΔΔ {:+.3} 個百分點｜模型內勝率 ΔΔ {:+.3} 個百分點｜完整配對 seed {}",
                first.label(),
                second.label(),
                share_contrast,
                probability_contrast * 100.0,
                paired.len()
            );
        }
    }
    println!("ΔΔ 是四個配對角點的差中差，用於檢查兩項倍率同時變動時是否偏離可加效果；這是模型內方向性檢查，不代表真實選舉因果或校準係數。每個 seed 執行 84 場，模式最多 100 個 seed。");

    if let Some(path) = options.batch_log.as_deref() {
        save_batch_log(path, options, role, region, &records)?;
        println!("逐場原始結果已追加至：{path}");
    }
    Ok(())
}

fn interaction_corners(
    records: &[BatchRecord],
    first: SensitivityParameter,
    second: SensitivityParameter,
) -> HashMap<u64, [Option<&BatchRecord>; 4]> {
    let mut corners_by_seed = HashMap::new();
    for record in records.iter().filter(|record| {
        record.sensitivity_parameter == first.key()
            && record.sensitivity_parameter_2 == second.key()
    }) {
        let first_high = (record.multiplier - 1.2).abs() < f64::EPSILON;
        let second_high = (record.multiplier_2 - 1.2).abs() < f64::EPSILON;
        let corner = match (first_high, second_high) {
            (false, false) => 0,
            (false, true) => 1,
            (true, false) => 2,
            (true, true) => 3,
        };
        corners_by_seed.entry(record.seed).or_insert([None; 4])[corner] = Some(record);
    }
    corners_by_seed
}

fn factorial_interaction(low_low: f64, low_high: f64, high_low: f64, high_high: f64) -> f64 {
    high_high - high_low - low_high + low_low
}

fn batch_metrics(group: &[&BatchRecord]) -> BatchMetrics {
    let count = group.len() as f64;
    let mut shares: Vec<_> = group
        .iter()
        .map(|record| record.player_final_share)
        .collect();
    shares.sort_by(f64::total_cmp);
    BatchMetrics {
        seed_win_rate: group.iter().filter(|record| record.player_won).count() as f64 / count,
        mean_model_probability: group
            .iter()
            .map(|record| record.player_win_probability)
            .sum::<f64>()
            / count,
        mean_final_share: group
            .iter()
            .map(|record| record.player_final_share)
            .sum::<f64>()
            / count,
        mean_signed_margin: group.iter().map(|record| record.signed_margin).sum::<f64>() / count,
        mean_objectives: group
            .iter()
            .map(|record| f64::from(record.objectives_completed))
            .sum::<f64>()
            / count,
        mean_exposed_tactics: group
            .iter()
            .map(|record| f64::from(record.exposed_tactics))
            .sum::<f64>()
            / count,
        mean_sanctioned_funds: group
            .iter()
            .map(|record| f64::from(record.sanctioned_funds))
            .sum::<f64>()
            / count,
        mean_budget_remaining: group
            .iter()
            .map(|record| f64::from(record.budget_remaining))
            .sum::<f64>()
            / count,
        mean_stamina_remaining: group
            .iter()
            .map(|record| record.stamina_remaining)
            .sum::<f64>()
            / count,
        mean_action_counts: std::array::from_fn(|index| {
            group
                .iter()
                .map(|record| f64::from(record.action_counts[index]))
                .sum::<f64>()
                / count
        }),
        mean_module_contributions: std::array::from_fn(|index| {
            group
                .iter()
                .map(|record| record.module_contribution_means[index])
                .sum::<f64>()
                / count
        }),
        shares,
    }
}

fn print_batch_metrics(label: &str, metrics: &BatchMetrics) {
    println!(
        "{}：seed 勝出率 {:.1}%｜模型內平均勝率 {:.1}%｜終局支持均值 {:.1}%（P10/P50/P90 {:.1}/{:.1}/{:.1}）｜平均票差 {:+.1}｜週任務 {:.1}/{}｜揭露 {:.2} 次／平均懲處 {:.2}｜剩餘預算 {:.1}／體力 {:.1}",
        label,
        metrics.seed_win_rate * 100.0,
        metrics.mean_model_probability * 100.0,
        metrics.mean_final_share,
        percentile(&metrics.shares, 0.10),
        percentile(&metrics.shares, 0.50),
        percentile(&metrics.shares, 0.90),
        metrics.mean_signed_margin,
        metrics.mean_objectives,
        CAMPAIGN_ROUNDS,
        metrics.mean_exposed_tactics,
        metrics.mean_sanctioned_funds,
        metrics.mean_budget_remaining,
        metrics.mean_stamina_remaining
    );
    println!(
        "  每場行動構成：正面 {:.1}／中立 {:.1}／負面 {:.1}；八軸平均加權貢獻：{}",
        metrics.mean_action_counts[0],
        metrics.mean_action_counts[1],
        metrics.mean_action_counts[2],
        StrategyAxis::ALL
            .iter()
            .zip(metrics.mean_module_contributions)
            .map(|(axis, contribution)| format!("{} {:+.2}", axis.label(), contribution))
            .collect::<Vec<_>>()
            .join("、")
    );
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let index = ((sorted.len() as f64 * p).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len().saturating_sub(1));
    sorted.get(index).copied().unwrap_or(0.0)
}

fn save_batch_log(
    path: &str,
    options: &Options,
    role: Side,
    region: Option<&RegionProfile>,
    records: &[BatchRecord],
) -> Result<(), String> {
    let log_path = Path::new(path);
    let exists = log_path.exists();
    let needs_header = !exists
        || log_path
            .metadata()
            .map_err(|error| error.to_string())?
            .len()
            == 0;
    if exists && !needs_header {
        let mut header = String::new();
        BufReader::new(std::fs::File::open(log_path).map_err(|error| error.to_string())?)
            .read_line(&mut header)
            .map_err(|error| error.to_string())?;
        if header.trim_end() != BATCH_LOG_HEADER {
            return Err(
                "批次紀錄欄位與目前版本不同，請指定新的 --batch-log 檔名以免混合格式。".to_string(),
            );
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("無法寫入批次紀錄：{error}"))?;
    if needs_header {
        writeln!(file, "{BATCH_LOG_HEADER}").map_err(|error| error.to_string())?;
    }
    let mode = if region.is_some() {
        "local"
    } else {
        "presidential"
    };
    let region_name = region.map(|value| value.name).unwrap_or("-");
    for record in records {
        let review = record.scorecard;
        let mut cells = vec![
            options.year.to_string(),
            mode.to_string(),
            tsv_cell(region_name),
            role.label().to_string(),
            record.style.label().to_string(),
            record.strategy.key().to_string(),
            record.seed.to_string(),
            record.sensitivity_parameter.to_string(),
            format!("{:.2}", record.multiplier),
            tsv_cell(&record.winner_candidate),
            tsv_cell(&record.winner_party),
            tsv_cell(&record.player_party),
            record.player_won.to_string(),
            format!("{:.4}", record.player_win_probability),
            format!("{:.2}", record.player_final_share),
            format!("{:.2}", record.signed_margin),
            record.objectives_completed.to_string(),
            record.budget_remaining.to_string(),
            format!("{:.2}", record.stamina_remaining),
            format!("{:.2}", record.trust),
            format!("{:.2}", record.mobilization),
            format!("{:.2}", review.policy),
            format!("{:.2}", review.event_response),
            format!("{:.2}", review.resource_management),
            format!("{:.2}", review.versatility),
            record.exposed_tactics.to_string(),
            record.sanctioned_funds.to_string(),
            record.sensitivity_parameter_2.to_string(),
            format!("{:.2}", record.multiplier_2),
        ];
        cells.extend(record.action_counts.iter().map(u16::to_string));
        cells.extend(
            StrategyAxis::ALL
                .iter()
                .zip(record.module_contribution_means)
                .map(|(_, contribution)| format!("{contribution:.4}")),
        );
        let action_composition = ActionId::ALL
            .iter()
            .zip(record.action_usage_counts)
            .filter(|(_, count)| *count > 0)
            .map(|(action, count)| format!("{}={count}", action.key()))
            .collect::<Vec<_>>()
            .join(";");
        cells.push(action_composition);
        cells.push(options.setup.difficulty.key().to_string());
        cells.push(options.setup.event_pack.key().to_string());
        cells.push(options.setup.timeline.key().to_string());
        cells.push(record.story_path.clone());
        writeln!(file, "{}", cells.join("\t")).map_err(|error| error.to_string())?;
    }
    file.flush().map_err(|error| error.to_string())?;
    Ok(())
}

fn tsv_cell(value: &str) -> String {
    value.replace(['\t', '\r', '\n'], " ")
}

fn print_action_outcome(
    game: &GameState,
    outcome: &project_twn::sim::ActionOutcome,
    staffer_mode: bool,
) {
    let candidate_name = &game.candidates[game.player_index].name;
    println!(
        "✓ 第 {} 週・第 {} 天 [{}]【已選】{}：{} 支持度 {:.2}% → {:.2}%（實際變化 {:+.2} 點；引擎預估 {:+.2} 點｜預算 -{}、週補給 +{}、餘額 {}）",
        outcome.week,
        outcome.day,
        outcome.category.label(),
        outcome.action_name,
        candidate_name,
        outcome.share_before,
        outcome.share_after,
        outcome.share_after - outcome.share_before,
        outcome.expected_support_delta,
        outcome.cost,
        outcome.weekly_funding,
        outcome.budget_after
    );
    if let Some(event) = outcome.event_title {
        println!("  對應公共議題：{event}");
    }
    if let Some(note) = &outcome.coordination_note {
        println!("  {note}");
    }
    if let Some(note) = &outcome.tactic_note {
        println!(
            "  {note}（本次行動預估揭露率 {:.0}%）",
            outcome.detection_probability * 100.0
        );
    }
    if let Some(title) = outcome.objective_title {
        let completion = if outcome.objective_completed && outcome.objective_reward > 0 {
            format!("；達成，預算獎勵 +{}", outcome.objective_reward)
        } else if outcome.objective_completed {
            "；已達成，但預算已達上限".to_string()
        } else {
            String::new()
        };
        println!(
            "  本週任務 {title}：{}/{}{}",
            outcome.objective_progress, outcome.objective_target, completion
        );
    }
    println!(
        "  團隊信任 {:.1}/100，動員指數 {:.1}/100，體力 {:.1}/100；地面調度量能 {:.0}/55",
        outcome.trust_after,
        outcome.turnout_after,
        outcome.stamina_after,
        outcome.field_capacity_after
    );
    if staffer_mode {
        if let Some(response) = &outcome.rival_response {
            println!(
                "  對手回應（遊戲模型）：{} 支持度 {:.2}% → {:.2}%（{:+.2} 點）；競爭盤勢也會隨每日行動變化。",
                response.candidate_name,
                response.share_before,
                response.share_after,
                response.share_after - response.share_before
            );
        }
        println!("  劇情推進：{}", staffer_action_reaction(outcome));
    } else {
        if let Some(response) = &outcome.rival_response {
            println!(
                "  對手回應（遊戲模型）：{} 支持度 {:.2}% → {:.2}%（{:+.2} 點）。",
                response.candidate_name,
                response.share_before,
                response.share_after,
                response.share_after - response.share_before
            );
        }
        println!("  候選人劇情：{}", candidate_action_reaction(outcome));
    }
    if outcome.day == DAYS_PER_WEEK {
        print_weekly_campaign_review(game);
        print_poll_table(game);
    }
}

fn print_weekly_campaign_review(game: &GameState) {
    let Some(report) = game.latest_weekly_report() else {
        return;
    };
    println!("\n【第 {} 週戰情回顧｜遊戲模型值】", report.week);
    for (index, candidate) in game.candidates.iter().enumerate() {
        let ending = report.ending_polls.get(index).copied().unwrap_or(0.0);
        let delta = report.candidate_deltas.get(index).copied().unwrap_or(0.0);
        let marker = if index == game.player_index {
            "▶"
        } else {
            " "
        };
        println!(
            "{marker} {}：{ending:.2}%（本週 {delta:+.2} 點）",
            candidate.name
        );
    }
    if let Some((leader_index, leader_share)) = report
        .ending_polls
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
    {
        let player_share = report
            .ending_polls
            .get(game.player_index)
            .copied()
            .unwrap_or(0.0);
        let margin = leader_share - player_share;
        if margin.abs() < 0.005 {
            println!("盤勢：玩家候選人與目前最高者並列。");
        } else if leader_index == game.player_index {
            println!(
                "盤勢：玩家候選人目前居首；與次高者差 {:.2} 點。",
                margin.max(0.0)
            );
        } else {
            println!(
                "盤勢：{} 暫居首位；玩家候選人落後 {:.2} 點。",
                game.candidates[leader_index].name, margin
            );
        }
    }
    println!(
        "團隊信任：{:.1}/100（本週 {:+.1}）；政府施政滿意度情境值：{:.1}/100（本週 {:+.1}）。",
        report.ending_trust,
        report.trust_delta,
        report.ending_administration_approval,
        report.administration_approval_delta
    );
    if !report.player_segment_deltas.is_empty() {
        let strongest = report
            .player_segment_deltas
            .iter()
            .enumerate()
            .max_by(|(_, left), (_, right)| left.total_cmp(right));
        let weakest = report
            .player_segment_deltas
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| left.total_cmp(right));
        if let (Some((up_index, up_delta)), Some((down_index, down_delta))) = (strongest, weakest) {
            if up_index == down_index {
                if let Some(segment) = game.voter_segments.get(up_index) {
                    println!(
                        "分眾變化：各遊戲原型接近持平，{} 約 {:+.2} 點（原型並非人口占比）。",
                        segment.name, up_delta
                    );
                }
            } else if let (Some(up_segment), Some(down_segment)) = (
                game.voter_segments.get(up_index),
                game.voter_segments.get(down_index),
            ) {
                println!(
                    "分眾變化較大：{} {:+.2} 點；{} {:+.2} 點（遊戲原型，非人口占比）。",
                    up_segment.name, up_delta, down_segment.name, down_delta
                );
            }
        }
    }
    println!(
        "本週任務「{}」：{}。支持度、分眾與施政滿意度均為遊戲狀態，不是實際民調或選舉預測。\n",
        report.objective_title,
        if report.objective_completed {
            "已完成"
        } else {
            "未完成"
        }
    );
}

fn staffer_action_reaction(outcome: &project_twn::sim::ActionOutcome) -> &'static str {
    use ActionId::*;

    if outcome.negative_tactic_detected {
        return "查核資料與完整紀錄同時送進新聞室，原本想搶下的版面變成連夜說明；團隊必須承受支持與信任下滑。";
    }
    match outcome.id {
        CampaignHeadquarters => match outcome.week {
            1..=3 => "總部的白板終於掛上第一張行程表，林予安要求每個承諾都要有負責人和回覆期限。",
            _ => "總部遲來的燈光亮到深夜；團隊得先補上流程缺口，才追得上對手已排好的地方行程。",
        },
        SupportersAssociation => "後援會開始接起地方電話，陳秀惠把尚未完成的陳情也交進聯絡清單，提醒團隊支持者不是一次性名單。",
        VolunteerCanvassing | GroundTeamLeaflets | StreetSweep | TurnoutMobilization => {
            if outcome.week >= 9 {
                "志工把最後一輪回報貼上戰情牆；名單多了幾個願意出門的人，也留下尚未找到的住戶。"
            } else {
                "路口行程結束後，志工把居民追問寫進回報表；方致遠的團隊也公布了新的掃街路線。"
            }
        }
        CommunityTour
        | LegalAssembly
        | ReligiousCommunityVisit
        | PilgrimageParticipation
        | MassAttendance => {
            "拜會結束，地方夥伴沒有替候選人背書，只要求把共同關心的事和彼此不同的立場都說清楚。"
        }
        VulnerableCommunityCare | DisasterResponseVisit | PublicServices | CrisisResponse => {
            "服務窗口回報已接到第一批需求，但仍有個案等待；林予安把候選人行程往後挪，要求先確認誰能接手。"
        }
        PolicyDelivery | PolicyForum | PolicyPresentation | Debate => {
            if outcome.week >= 9 {
                "政見回答被剪成短片流傳，周以晴仍追問成本與期限；團隊把完整資料和未完成項目一併公開。"
            } else {
                "政策組把下一版成本表交給候選人，方致遠的幕僚已在記者群組追問財源與時程。"
            }
        }
        TransparencyReport | PublicMessage | MediaInterview | FactCheckResponse | OversightPetition => {
            "周以晴把查證結果與待確認欄位並排刊出；團隊可以更正錯誤，卻不能只挑有利的半段回應。"
        }
        CoalitionTalks | PollCoordination => {
            if outcome.coordination_note.is_some() {
                "協商結果傳回總部，合作方仍保留自己的招牌與主張；共同動員不代表分歧已經消失。"
            } else {
                "協商沒有立刻定案。林予安把下一次會談留在行事曆，並要求今天先把各方不能退讓的條件寫明。"
            }
        }
        RestAndPrepare => "總部暫時關掉一半燈光，候選人補足體力，幕僚留下來整理明天必須回覆的問題。",
        FabricatedPoll | AnonymousSmear | CoordinatedOpinionWar => {
            "查核尚未確認來源，團隊沒有把暫時無事當成安全；完整紀錄已交由負責人保全。"
        }
    }
}

fn candidate_action_reaction(outcome: &project_twn::sim::ActionOutcome) -> &'static str {
    use ActionId::*;

    if outcome.negative_tactic_detected {
        return "記者會上，查核團體播出完整紀錄；你原想掌握的議題被迫讓位給道歉與說明，團隊連夜重排行程。";
    }
    match outcome.id {
        CampaignHeadquarters => match outcome.week {
            1..=3 => "你站在新成立的總部門口，看著行程表第一次貼滿整面牆；林予安提醒，真正的考驗是每張承諾都有人追進度。",
            _ => "你走進才剛整理好的總部，發現對手早已跑完幾輪地方行程；團隊得用更精準的排程追回時間。",
        },
        SupportersAssociation => "後援會把第一批聯絡名單交到你手上；其中有熱情支持者，也有等著你回覆的陳情人。",
        VolunteerCanvassing | GroundTeamLeaflets | StreetSweep | TurnoutMobilization => {
            if outcome.week >= 9 {
                "最後一輪掃街結束，你聽見支持者問投票交通，也聽見還沒決定的人追問政見；兩種聲音都進了回報表。"
            } else {
                "路口行程結束，你把居民追問記在袖珍筆記本裡；轉角處，方致遠的團隊正架起另一個宣傳點。"
            }
        }
        CommunityTour | LegalAssembly | ReligiousCommunityVisit | PilgrimageParticipation | MassAttendance => {
            "拜會結束，地方夥伴沒有替你背書，只要求公開共同關心的事與彼此不同的立場；鏡頭收起後，談話才真正開始。"
        }
        VulnerableCommunityCare | DisasterResponseVisit | PublicServices | CrisisResponse => {
            "你先聽完現場需求才接受媒體訪問；仍有個案等候處理，團隊把後續聯絡窗口一併公布。"
        }
        PolicyDelivery | PolicyForum | PolicyPresentation | Debate => {
            if outcome.week >= 9 {
                "政見回答被剪成短片傳開，周以晴仍在台下追問成本與期限；你把完整資料和未完成項目一起交給媒體。"
            } else {
                "政策組把成本表推到你面前，記者已經準備追問財源；你得在簡短答覆和完整交代之間找到節奏。"
            }
        }
        TransparencyReport | PublicMessage | MediaInterview | FactCheckResponse | OversightPetition => {
            "周以晴把查證結果和待確認欄位並排刊出；你可以更正錯誤，但不能只挑有利的一半回答。"
        }
        CoalitionTalks | PollCoordination => {
            if outcome.coordination_note.is_some() {
                "協調結果傳回總部，合作方保留自己的招牌與主張；你知道共同出席不等於所有分歧都已消失。"
            } else {
                "協商沒有立即定案。你把下一次會談留在行事曆，並請團隊先寫清楚雙方不能退讓的條件。"
            }
        }
        RestAndPrepare => "你離開鏡頭休息一段時間，幕僚留下來整理明天的提問；候選人的體力也是這場選戰的資源。",
        FabricatedPoll | AnonymousSmear | CoordinatedOpinionWar => {
            "目前沒有查核揭露，但周以晴已要求提供來源；你知道這條捷徑可能在下一個新聞週期反噬。"
        }
    }
}

fn print_final(game: &mut GameState) -> ElectionOutcome {
    let result = game.finish();
    let winner = &game.candidates[result.winner_index];
    println!("\n=== 情境開票（單次模擬）===");
    for (index, candidate) in game.candidates.iter().enumerate() {
        let mark = if index == result.winner_index {
            "★"
        } else {
            " "
        };
        println!(
            "{mark} {:<22} {:>5.2}%",
            candidate.name, result.shares[index]
        );
    }
    println!("\n情境勝選者：{}（{}）", winner.name, winner.party);
    println!(
        "玩家在 1,200 次小幅終盤誤差抽樣中的勝選率：{:.1}%",
        result.player_win_probability * 100.0
    );
    println!("這是遊戲隨機模型的結果，不是民調預測或真實選舉結果。\n");
    println!(
        "每週任務達成：{}/{}",
        game.objectives_completed(),
        CAMPAIGN_ROUNDS
    );
    println!(
        "選戰誠信紀錄：遭查核揭露 {} 次、累計懲處資源 {}；終局地面調度量能 {:.0}/55。",
        game.exposed_tactics, game.sanctioned_funds, game.field_capacity
    );
    println!("=== 戰役操作評分（0–100；非遊戲品質評測）===");
    for (axis, score) in game.scorecard().axes() {
        let bars = "█".repeat((score / 5.0).round() as usize);
        println!("  {axis:<8} {score:>5.1} {bars}");
    }
    let review = game.scorecard();
    let balanced = review.axes().iter().all(|(_, score)| *score >= 65.0);
    println!(
        "稱號：{}",
        if balanced {
            "全能競選團隊"
        } else if review.versatility >= 70.0 {
            "議題百變團隊"
        } else if review.mobilization >= 65.0 {
            "組織動員團隊"
        } else {
            "持續累積中"
        }
    );
    result
}

fn run_primary(seed: u64, auto: bool) -> Result<CampaignStyle, String> {
    let mut primary = PrimaryContest::new(seed ^ 0xA11C_E5ED);
    println!("\n=== 黨內初選：提名之爭 ===");
    println!("三位虛構提名路線：政策派、組織派、溝通派。先完成 {PRIMARY_ROUNDS} 回合初選，再進入十二週大選。\n");
    print_primary_polls(&primary);
    while !primary.is_finished() {
        println!(
            "\n--- 初選第 {} / {} 回合 ---",
            primary.round + 1,
            PRIMARY_ROUNDS
        );
        println!(
            "  1) 政策論壇　2) 地方組織行程　3) 公開溝通活動　4) 對比／互比式民調　5) 初選整合協商"
        );
        let choice = if auto {
            [
                PrimaryChoice::PolicyDebate,
                PrimaryChoice::OrganizerVisit,
                PrimaryChoice::ComparisonPoll,
                PrimaryChoice::UnityNegotiation,
            ][primary.round as usize]
        } else {
            loop {
                print!("選擇策略：");
                io::stdout().flush().map_err(|error| error.to_string())?;
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .map_err(|error| error.to_string())?;
                let selected = match input.trim() {
                    "1" => Some(PrimaryChoice::PolicyDebate),
                    "2" => Some(PrimaryChoice::OrganizerVisit),
                    "3" => Some(PrimaryChoice::CommunicationForum),
                    "4" => Some(PrimaryChoice::ComparisonPoll),
                    "5" => Some(PrimaryChoice::UnityNegotiation),
                    _ => None,
                };
                if let Some(selected) = selected {
                    break selected;
                }
                println!("請輸入 1–5。");
            }
        };
        primary.apply(choice)?;
        println!("採取：{}", choice.label());
        print_primary_polls(&primary);
    }
    let nominee = primary.nominee_index();
    let style = primary.nominee_style();
    println!(
        "\n初選提名：{}（支持度 {:.1}%），以此路線進入大選。\n",
        PrimaryContest::name(nominee),
        primary.polls[nominee]
    );
    Ok(style)
}

fn print_primary_polls(primary: &PrimaryContest) {
    println!("對比式初選支持度");
    for index in 0..3 {
        println!(
            "  {:<5} {:>5.1}%",
            PrimaryContest::name(index),
            primary.polls[index]
        );
    }
    println!("互比式初選（未入互比者支持各半轉移；簡化情境值）");
    for (left, right, left_share) in primary.head_to_head() {
        println!(
            "  {} {:.1}% : {} {:.1}%",
            PrimaryContest::name(left),
            left_share,
            PrimaryContest::name(right),
            100.0 - left_share
        );
    }
}

fn print_regions() {
    println!("可選虛構地方情境（政黨、地名與前輪支持設定皆為遊戲內容）\n");
    for (index, region) in REGIONS.iter().enumerate() {
        println!(
            "  {:>2}. {:<5} {:<8} 上屆：{}　焦點：{}",
            index + 1,
            region.key,
            region.name,
            region.previous_winner_party,
            region.focus_label
        );
    }
}

fn print_parties() {
    println!("可選政黨／地方聯盟情境範本（名稱、描述與策略軸線均為虛構遊戲設定）\n");
    for party in PARTY_PROFILES {
        println!(
            "  {:<12} {:<18} {}",
            party.name, party.camp, party.description
        );
    }
    println!("\n地方模式使用：--year 2026 --region 地區 --role challenger --party 政黨名稱");
}

fn save_campaign_log(
    path: &str,
    game: &GameState,
    result: &ElectionOutcome,
    seed: u64,
    strategy: StrategyProfile,
    staffer_mode: bool,
) -> Result<(), String> {
    let log_path = Path::new(path);
    let exists = log_path.exists();
    let needs_header = !exists
        || log_path
            .metadata()
            .map_err(|error| error.to_string())?
            .len()
            == 0;
    if exists && !needs_header {
        let mut header = String::new();
        BufReader::new(std::fs::File::open(log_path).map_err(|error| error.to_string())?)
            .read_line(&mut header)
            .map_err(|error| error.to_string())?;
        if header.trim_end() != CAMPAIGN_LOG_HEADER {
            return Err(
                "戰役紀錄欄位與目前版本不同，請指定新的 --log 檔名以免混合格式。".to_string(),
            );
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("無法寫入戰役紀錄：{error}"))?;
    if needs_header {
        writeln!(file, "{CAMPAIGN_LOG_HEADER}").map_err(|error| error.to_string())?;
    }
    let winner = &game.candidates[result.winner_index];
    let review = game.scorecard();
    writeln!(
        file,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        game.campaign_year,
        if game.region.is_some() {
            "local"
        } else {
            "presidential"
        },
        game.region
            .map(|region| region.name)
            .unwrap_or("-")
            .replace('\t', " "),
        game.side.label(),
        game.style.label(),
        strategy.key(),
        seed,
        winner.party,
        result.player_win_probability,
        game.objectives_completed(),
        review.policy,
        review.trust,
        review.mobilization,
        review.event_response,
        review.resource_management,
        review.versatility,
        game.exposed_tactics,
        game.sanctioned_funds,
        game.setup.difficulty.key(),
        game.setup.event_pack.key(),
        game.setup.timeline.key(),
        if staffer_mode {
            "-".to_string()
        } else {
            game.story_progress.summary()
        },
        if staffer_mode {
            game.staffer_progress.summary()
        } else {
            "-".to_string()
        }
    )
    .map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())?;
    Ok(())
}

fn print_scenarios() {
    println!("虛構選戰情境年表（人物、政黨、地名、票數與事件皆為本作設定；非史實或預測）\n");
    for election in ELECTIONS {
        println!(
            "{} ({})　情境執政黨：{}",
            election.year, election.date, election.incumbent_party
        );
        for candidate in election.candidates {
            println!(
                "  {:<8} {:<7} {:>7}  {:>5.2}%",
                candidate.president, candidate.party, candidate.votes, candidate.share_pct
            );
        }
    }
    println!("\n虛構議會席次情境");
    for snapshot in LEGISLATURE_SNAPSHOTS {
        println!("  {}：{}", snapshot.year, snapshot.summary);
    }
    println!("\n公共議題事件卡（角色與事件皆為虛構情境）");
    for event in EVENTS {
        println!("  {}｜{}：{}", event.year, event.title, event.description);
    }
    println!("\n選民原型（遊戲權重，不是人口統計或實際占比）");
    for segment in VOTER_SEGMENTS {
        println!(
            "  {:<20} {:>4.0}%｜{}",
            segment.name, segment.weight_pct, segment.description
        );
    }
    println!("\n所有年表票數及比例均為遊戲參數，玩家可見人物、團體與事件不對應特定真實對象。\n");
}

fn print_usage() {
    println!(
        "Project-TWN — 臺灣選戰模擬器

使用方式：
  project-twn --scenarios [--history 相容別名]
  project-twn --regions
  project-twn --parties
  project-twn --roster-editor 名冊.tsv
  project-twn --recall [--recall-side initiator|opposition] [--recall-template fictional|low-turnout|concentrated|high-mobilization] [--seed 整數] [--auto] [--save 存檔.tsv]
  project-twn --simulate [年份] [--region 地區] [--party 政黨] [--role incumbent|challenger]
      [--style balanced|policy|organizer|communicator] [--strategy balanced|persuasion|issue|mobilization|trust|sustainable|hardball]
      [--difficulty story|standard|hard] [--event-pack all|democracy|cross-strait|local|media] [--timeline weekly|date]
      [--primary | --staffer] [--roster 名冊.tsv] [--seed 整數] [--auto] [--log 檔案.tsv] [--save 存檔.tsv]
  project-twn --load 存檔.tsv
  project-twn --batch 場數 --role incumbent|challenger [--style 風格] [--strategy 策略]
      [--sweep-styles | --sweep-strategies | --sensitivity | --interaction-sensitivity]
      [--year 年份] [--region 地區] [--party 政黨] [--seed 起始值] [--batch-log 原始跑次.tsv]

範例：
  cargo run -- --simulate 2024 --role challenger --style organizer --seed 42
  cargo run -- --simulate 2024 --role challenger --difficulty story --event-pack democracy --timeline date
  cargo run -- --year 2024 --role challenger --staffer --seed 42
  cargo run -- --year 2024 --role challenger --save campaign.tsv
  cargo run -- --load campaign.tsv
  cargo run -- --roster-editor roster.tsv
  cargo run -- --recall --recall-side initiator --recall-template high-mobilization --seed 42
  cargo run -- --year 2026 --region 霧川市 --role challenger --party 山海自治黨 --strategy issue --seed 7 --auto
  cargo run -- --batch 1000 --year 2024 --role challenger --seed 1000 --sweep-styles --batch-log batch.tsv
  cargo run -- --batch 100 --year 2024 --role challenger --seed 1000 --sweep-strategies --batch-log strategy.tsv
  cargo run -- --batch 100 --year 2024 --role challenger --seed 1000 --sensitivity --batch-log sensitivity.tsv
  cargo run -- --batch 10 --year 2024 --role challenger --seed 1000 --interaction-sensitivity --batch-log interactions.tsv

年份範圍：1996–2026。--scenarios 顯示虛構年表與事件卡（--history 為相容別名）；地方模式可用 --regions 檢視 22 個虛構地區，--parties 檢視虛構政黨範本。
競選為 12 週、每週 7 天，共 84 次每日行動決策；每週有資金補給。--primary 先進行四回合黨內初選。
--staffer 開啟幕僚模式：設定競選主軸，並在六個關鍵週處理陳情、地方協調、事件回應、支持度檢視與最後布局；幕僚決策不占每日行動，但會消耗預算及影響遊戲資源。此單場模式目前不支援 --batch 或 --primary。
--recall 開啟人民罷免模式：在罷免提議／連署方或反對罷免方之間選擇，進行 14 日提議、60 日連署與 28 日投票宣傳；可選虛構的低投票參與、集中支持或高強度動員盤型。本模式不含彈劾；遊戲中人物與事件皆採虛構合成，不對應真實對象。不實／死亡連署是違規高風險行動，不計入有效連署，遭查核會受遊戲內懲處。
--roster-editor 會建立 2–5 人的虛構候選人名冊 TSV；--roster 可將姓名、虛構政黨及起始支持度套用到單場戰役。名冊只接受內建虛構政黨，須明確指定唯一一位玩家候選人。
--save 會在互動戰役開始及每次有效決策後原子更新版本化 TSV 存檔；--load 會依固定 seed 重播決策紀錄後接續遊玩，適用候選人、幕僚及罷免模式。自動示範與批次模式不支援存檔；舊版或未知版本存檔會拒絕載入。
難度可選故事／標準／挑戰；事件包可篩選制度參與、區域安全、地方民生或媒體信任等虛構議題；--timeline date 會將虛構事件的遊戲日期錨點映射到 12 週戰役日曆，沒有錨點的週次留白。
省略單場模式的 --role 會詢問陣營；--auto 依所選策略逐日選擇引擎排序最高的合法行動。--log 記錄單場戰役；--batch-log 記錄批次每一 seed 的原始跑次結果。
策略引擎將各合法行動的支持度估值、事件／地方契合、信任、動員、每週任務、資源續航、誠信風險及組織基礎拆成可組合模組；共通行動涵蓋總部／後援會、地面動員、公共溝通與社區服務，另有會受查核懲處的高風險負面行動。
--strategy 選擇七種權重組合，互動模式顯示行動分類、排序及預估揭露風險，--auto 依排序採取行動。
策略分數是遊戲啟發式，支持度是模型一步期望值；執行後另外顯示含 seed 波動與對手反應的實際變化，兩者都不是民調或選舉預測。
批次最多 10,000 個 seed；--sweep-styles 會以同一 seed 範圍比較四種能力風格；--sweep-strategies 以同一 seed 比較七種策略模組組合（最多 1,000 個 seed），並回報揭露／懲處、剩餘預算、行動構成與八軸平均加權貢獻；--sensitivity 以配對 seed 對單一模型參數做 ±20% 測試（最多 1,000 個 seed）；--interaction-sensitivity 以 2×2 配對檢查兩項參數的交互作用（每個 seed 84 場，最多 100 個 seed）。策略貢獻是排序分數分解；批次數值是遊戲情境結果，不是民調或選舉預測。"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_menu_categories_partition_all_actions() {
        let assignments: Vec<_> = ActionId::ALL
            .iter()
            .filter(|id| {
                ActionMenuCategory::ALL
                    .iter()
                    .filter(|category| category.includes(**id))
                    .count()
                    == 1
            })
            .collect();
        assert_eq!(assignments.len(), ActionId::COUNT);
        let largest_group = ActionMenuCategory::ALL
            .iter()
            .map(|category| {
                ActionId::ALL
                    .iter()
                    .filter(|id| category.includes(**id))
                    .count()
            })
            .max()
            .unwrap();
        assert_eq!(largest_group, 8);
    }

    #[test]
    fn percentile_uses_nearest_rank_and_handles_empty_data() {
        let values = [10.0, 20.0, 30.0, 40.0];
        assert_eq!(percentile(&values, 0.10), 10.0);
        assert_eq!(percentile(&values, 0.50), 20.0);
        assert_eq!(percentile(&values, 0.90), 40.0);
        assert_eq!(percentile(&[], 0.50), 0.0);
    }

    #[test]
    fn factorial_interaction_is_zero_for_additive_effects() {
        let low_low = 10.0;
        let low_high = 13.0;
        let high_low = 12.0;
        let high_high = 15.0;
        assert_eq!(
            factorial_interaction(low_low, low_high, high_low, high_high),
            0.0
        );
    }

    #[test]
    fn factorial_interaction_detects_joint_effect() {
        assert_eq!(factorial_interaction(0.0, 0.0, 0.0, 4.0), 4.0);
    }

    #[test]
    fn batch_autopilot_is_reproducible_for_a_fixed_seed_and_style() {
        let mut left = GameState::new_with_style(2024, Side::Challenger, 42, CampaignStyle::Policy);
        let mut right =
            GameState::new_with_style(2024, Side::Challenger, 42, CampaignStyle::Policy);
        let left_summary = run_auto(&mut left, false, StrategyProfile::Balanced);
        let right_summary = run_auto(&mut right, false, StrategyProfile::Balanced);
        assert_eq!(left_summary.action_counts, right_summary.action_counts);
        assert_eq!(
            left_summary.action_usage_counts,
            right_summary.action_usage_counts
        );
        assert_eq!(left_summary.actions_taken, 84);
        assert_eq!(left_summary.action_counts.iter().sum::<u16>(), 84);
        assert_eq!(left_summary.action_usage_counts.iter().sum::<u16>(), 84);
        assert!(left_summary
            .mean_module_contributions()
            .iter()
            .all(|contribution| contribution.is_finite()));
        assert_eq!(left.polls, right.polls);
        assert_eq!(left.finish(), right.finish());
        assert_eq!(left.objectives_completed(), right.objectives_completed());
    }

    #[test]
    fn staffer_autopilot_replays_all_briefings_for_a_fixed_seed() {
        let mut left = GameState::new(2024, Side::Challenger, 909);
        let mut right = GameState::new(2024, Side::Challenger, 909);
        let left_summary = run_auto_staffer(&mut left, false, StrategyProfile::Balanced);
        let right_summary = run_auto_staffer(&mut right, false, StrategyProfile::Balanced);
        assert_eq!(
            left_summary.action_usage_counts,
            right_summary.action_usage_counts
        );
        assert_eq!(left.polls, right.polls);
        assert_eq!(left.staffer_progress, right.staffer_progress);
        assert_eq!(left.staffer_progress.completed_briefings(), 6);
        assert_eq!(left.round, CAMPAIGN_ROUNDS);
        assert!(left.validate().is_ok());
    }
}
