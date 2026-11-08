use project_twn::campaign::{CampaignDifficulty, CampaignSetup, EventPack, EventTimeline};
use project_twn::history::{
    find_party, find_region, PartyProfile, RegionProfile, ELECTIONS, EVENTS, LEGISLATURE_SNAPSHOTS,
    PARTY_PROFILES, REGIONS, VOTER_SEGMENTS,
};
use project_twn::primary::{PrimaryChoice, PrimaryContest, PRIMARY_ROUNDS};
use project_twn::sim::{
    ActionId, CampaignScorecard, CampaignStyle, ElectionOutcome, GameError, GameState,
    ModelParameters, SensitivityParameter, Side, StoryOutcome, TacticCategory, CAMPAIGN_ROUNDS,
    DAYS_PER_WEEK,
};
use project_twn::story::coherence_bonus;
use project_twn::strategy::{
    StrategyAxis, StrategyEngine, StrategyProfile, StrategyRecommendation,
};
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const CAMPAIGN_LOG_HEADER: &str = "year\tmode\tregion\trole\tstyle\tstrategy\tseed\twinner\tplayer_win_probability\tobjectives_completed\tpolicy\ttrust\tmobilization\tevent_response\tresource_management\tversatility\texposed_tactics\tsanctioned_funds\tdifficulty\tevent_pack\ttimeline\tstory_path";
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
    history: bool,
    list_regions: bool,
    list_parties: bool,
    region: Option<String>,
    party: Option<String>,
    setup: CampaignSetup,
    primary: bool,
    log: Option<String>,
    batch: Option<u32>,
    sweep_styles: bool,
    sweep_strategies: bool,
    sensitivity: bool,
    interaction_sensitivity: bool,
    batch_log: Option<String>,
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
    let options = parse_args()?;
    if options.history {
        print_history();
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

    if let Some(runs) = options.batch {
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
    print_intro(&game, options.seed, options.strategy);

    if options.auto {
        run_auto(&mut game, true, options.strategy);
    } else {
        run_interactive(&mut game, options.strategy)?;
    }
    let result = print_final(&mut game);
    if let Some(path) = options.log.as_deref() {
        save_campaign_log(path, &game, &result, options.seed, options.strategy)?;
        println!("本場紀錄已追加至：{path}");
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
        return Err("地方首長模式目前是 2026 前瞻情境，請使用 --year 2026。".to_string());
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
            return Err("執政方 --party 必須符合該地 2022 上屆首長政黨；若扮演其他政黨請選 --role challenger。".to_string());
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

fn parse_args() -> Result<Options, String> {
    let mut args = env::args().skip(1);
    let mut options = Options {
        year: 2026,
        role: None,
        style: CampaignStyle::Balanced,
        strategy: StrategyProfile::Balanced,
        seed: 2026,
        auto: false,
        history: false,
        list_regions: false,
        list_parties: false,
        region: None,
        party: None,
        setup: CampaignSetup::default(),
        primary: false,
        log: None,
        batch: None,
        sweep_styles: false,
        sweep_strategies: false,
        sensitivity: false,
        interaction_sensitivity: false,
        batch_log: None,
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            "--history" => options.history = true,
            "--regions" => options.list_regions = true,
            "--parties" => options.list_parties = true,
            "--auto" | "--demo" => options.auto = true,
            "--primary" => options.primary = true,
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

fn print_intro(game: &GameState, seed: u64, strategy: StrategyProfile) {
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
            "選舉基準：{} 年總統選舉，初始候選人支持度以該屆實際得票率為起點，不是當時民調。",
            game.baseline_year
        );
    }
    if game.region.is_none() && game.campaign_year != game.baseline_year {
        println!("注意：本場使用政黨情境代表，不是該年度的實際候選人；支持度沿用最近一次總統票作遊戲起點，不代表當期民調或選舉結果。");
    }
    if game.campaign_year == 2026 {
        if game.region.is_some() {
            println!("2026地方選舉模式：2022年各地上屆首長政黨作為基準情境；2026投票日為 11-28，截至 11-08 尚無開票結果。" );
        } else {
            println!("2026全國代理情境：以2024總統票作全國基準，不是地方候選人或縣市民調；地方投票日為 2026-11-28。" );
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
    println!("民調口徑：本作目前只有候選人選舉支持度，尚未建模國政／政府施政滿意度民調；團隊信任指數也不是政府支持度。\n");
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
    print_poll_table(game);
    println!("\n行動對候選人支持度的效果會受團隊信任、議題契合、固定 seed 的小幅波動，以及對手回應影響。\n");
}

fn print_player_assignment(game: &GameState) {
    let player = &game.candidates[game.player_index];
    let context = if game.region.is_some() {
        "地方政黨情境代表"
    } else if game.campaign_year == game.baseline_year {
        "歷史選舉候選人"
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
    println!("候選人選舉支持度｜對比式情境（遊戲模型值；非實際民調，也不是國政施政滿意度）");
    for (index, candidate) in game.candidates.iter().enumerate() {
        let marker = if index == game.player_index {
            "▶"
        } else {
            " "
        };
        let bar = "█".repeat((game.polls[index] / 2.0).round() as usize);
        println!(
            "{marker} {:<22} {:>5.2}% {bar}",
            candidate.name, game.polls[index]
        );
    }
    println!("分眾支持度（原型權重／{player_name} 支持度）— 僅供觀察遊戲模型：");
    for segment in &game.voter_segments {
        let player_support = segment.polls.get(game.player_index).copied().unwrap_or(0.0);
        println!(
            "  {:<20} {:>4.0}%／{:>5.1}%",
            segment.name, segment.weight_pct, player_support
        );
    }
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

fn run_interactive(game: &mut GameState, strategy: StrategyProfile) -> Result<(), String> {
    let engine = StrategyEngine::new(strategy);
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
        if let Some(event) = game.current_event() {
            println!("議題焦點：{} — {}", event.title, event.description);
        }
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
        if let Some(beat) = game.current_story_beat() {
            println!("\n【多週情境｜第 {} 週】{}", beat.week, beat.title);
            println!("登場人物：{}", beat.character);
            println!("{}", beat.scene);
            println!("（人物與情境為虛構合成素材；選擇會改變遊戲資源與支持度，不耗用每日行動。）");
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
                    println!("本段情境暫不處理；後續故事仍會依目前進度展開。\n");
                    break;
                }
                match game.apply_story_choice(selection - 1) {
                    Ok(outcome) => {
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
        let recommendations = engine.rank_actions(game);
        let selected = select_interactive_action(&recommendations)?;
        let Some(action_id) = selected else {
            break;
        };
        match game.apply_action(action_id) {
            Ok(outcome) => print_action_outcome(game, &outcome),
            Err(message) => println!("{message}\n"),
        }
    }
    Ok(())
}

fn select_story_choice(choice_count: usize) -> Result<Option<usize>, String> {
    loop {
        print!("選擇解法（1–{choice_count}，0 暫不處理，q 結束本場）：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        let value = input.trim();
        if value.eq_ignore_ascii_case("q") {
            return Ok(None);
        }
        if let Ok(selection) = value.parse::<usize>() {
            if selection <= choice_count {
                return Ok(Some(selection));
            }
        }
        println!("請輸入有效的選項、0 或 q。\n");
    }
}

fn print_story_outcome(game: &GameState, outcome: &StoryOutcome) {
    let candidate_name = &game.candidates[game.player_index].name;
    println!(
        "✓ {}｜{}：{} 支持度 {:.2}% → {:.2}%（{:+.2} 點），團隊信任 {:.1}、動員 {:.1}、體力 {:.1}、預算 {}",
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
    if let Some(note) = outcome.note {
        println!("  {note}");
    }
}

fn select_interactive_action(
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

        println!("可選行動分類（依目前陣營與預算篩選）：");
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
        println!("  r) 直接執行全場策略推薦　q) 結束本場");
        print!("選分類：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        let choice = input.trim();
        if choice.eq_ignore_ascii_case("q") {
            return Ok(None);
        }
        if choice.eq_ignore_ascii_case("r") {
            return Ok(recommendations
                .first()
                .map(|recommendation| recommendation.preview.id));
        }
        let Ok(index) = choice.parse::<usize>() else {
            println!("請輸入分類編號、r 或 q。\n");
            continue;
        };
        let Some((category, options)) = categories.get(index.saturating_sub(1)) else {
            println!("沒有這個分類。\n");
            continue;
        };

        loop {
            println!("\n【{}】", category.label());
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
                    "  {}) [{}] {}｜預算 {}｜支持度 {:+.2}｜團隊信任 {:+.2}｜動員 {:+.2}｜體力 {:+.1}｜策略分 {:.2}{}{}",
                    option_index + 1,
                    recommendation.preview.category.label(),
                    recommendation.preview.action_name,
                    recommendation.preview.cost,
                    recommendation.preview.expected_support_delta,
                    recommendation.preview.trust_delta,
                    recommendation.preview.turnout_delta,
                    recommendation.preview.stamina_delta,
                    recommendation.score,
                    risk,
                    infrastructure
                );
            }
            print!("選行動編號（b 返回分類，q 結束）：");
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
                return Ok(None);
            }
            let Ok(option_index) = selected.parse::<usize>() else {
                println!("請輸入行動編號、b 或 q。\n");
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
    let engine = StrategyEngine::new(strategy);
    let mut summary = AutoRunSummary::default();
    while !game.is_finished() {
        if let Some(beat) = game.current_story_beat() {
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
            }
        }
        let Some(recommendation) = engine.rank_actions(game).into_iter().next() else {
            break;
        };
        let Ok(outcome) = game.apply_action(recommendation.preview.id) else {
            break;
        };
        summary.record(&recommendation);
        if show_progress {
            print_action_outcome(game, &outcome);
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

fn print_action_outcome(game: &GameState, outcome: &project_twn::sim::ActionOutcome) {
    let candidate_name = &game.candidates[game.player_index].name;
    println!(
        "✓ 第 {} 週・第 {} 天 [{}] {}：{} 支持度 {:.2}% → {:.2}%（實際變化 {:+.2} 點；引擎預估 {:+.2} 點｜預算 -{}、週補給 +{}、餘額 {}）",
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
    if outcome.day == DAYS_PER_WEEK {
        print_poll_table(game);
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
    println!("可選地方情境（上屆首長政黨依 2022 地方選舉；議題標籤是遊戲情境設定）\n");
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
    println!("可選第三勢力／政黨情境範本（不代表每一黨都會參加2026年選舉；軸線是策略參數）\n");
    for party in PARTY_PROFILES {
        println!(
            "  {:<12} {:<18} {}",
            party.name, party.camp, party.description
        );
    }
    println!("\n地方模式使用：--year 2026 --region 縣市 --role challenger --party 政黨名稱");
}

fn save_campaign_log(
    path: &str,
    game: &GameState,
    result: &ElectionOutcome,
    seed: u64,
    strategy: StrategyProfile,
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
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}",
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
        game.story_progress.summary()
    )
    .map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())?;
    Ok(())
}

fn print_history() {
    println!("臺灣總統副總統直選結果（中選會資料；列出得票率）\n");
    for election in ELECTIONS {
        println!(
            "{} ({})　當時執政黨：{}",
            election.year, election.date, election.incumbent_party
        );
        for candidate in election.candidates {
            println!(
                "  {:<8} {:<7} {:>7}  {:>5.2}%",
                candidate.president, candidate.party, candidate.votes, candidate.share_pct
            );
        }
    }
    println!("\n立法院關鍵席次節點");
    for snapshot in LEGISLATURE_SNAPSHOTS {
        println!("  {}：{}", snapshot.year, snapshot.summary);
    }
    println!("\n近代臺灣選戰與社會事件（事件說明為背景摘要，不是單一因果判斷）");
    for event in EVENTS {
        println!("  {}｜{}：{}", event.year, event.title, event.description);
    }
    println!("\n選民原型（遊戲權重，不是實際人口占比）");
    for segment in VOTER_SEGMENTS {
        println!(
            "  {:<20} {:>4.0}%｜{}",
            segment.name, segment.weight_pct, segment.description
        );
    }
    println!(
        "\n2026：地方公職人員選舉投票日為 2026-11-28；截至 2026-11-08 尚未投票，故不列結果。\n"
    );
}

fn print_usage() {
    println!(
        "Project-TWN — 臺灣選戰模擬器

使用方式：
  project-twn --history
  project-twn --regions
  project-twn --parties
  project-twn --simulate [年份] [--region 縣市] [--party 政黨] [--role incumbent|challenger]
      [--style balanced|policy|organizer|communicator] [--strategy balanced|persuasion|issue|mobilization|trust|sustainable|hardball]
      [--difficulty story|standard|hard] [--event-pack all|democracy|cross-strait|local|media] [--timeline weekly|date]
      [--primary] [--seed 整數] [--auto] [--log 檔案.tsv]
  project-twn --batch 場數 --role incumbent|challenger [--style 風格] [--strategy 策略]
      [--sweep-styles | --sweep-strategies | --sensitivity | --interaction-sensitivity]
      [--year 年份] [--region 縣市] [--party 政黨] [--seed 起始值] [--batch-log 原始跑次.tsv]

範例：
  cargo run -- --simulate 2024 --role challenger --style organizer --seed 42
  cargo run -- --simulate 2024 --role challenger --difficulty story --event-pack democracy --timeline date
  cargo run -- --year 2026 --region 臺北市 --role challenger --party 時代力量 --strategy issue --seed 7 --auto
  cargo run -- --batch 1000 --year 2024 --role challenger --seed 1000 --sweep-styles --batch-log batch.tsv
  cargo run -- --batch 100 --year 2024 --role challenger --seed 1000 --sweep-strategies --batch-log strategy.tsv
  cargo run -- --batch 100 --year 2024 --role challenger --seed 1000 --sensitivity --batch-log sensitivity.tsv
  cargo run -- --batch 10 --year 2024 --role challenger --seed 1000 --interaction-sensitivity --batch-log interactions.tsv

年份範圍：1996–2026。地方模式可用 --regions 檢視 22 個縣市；--parties 檢視政黨範本。
競選為 12 週、每週 7 天，共 84 次每日行動決策；每週有資金補給。--primary 先進行四回合黨內初選。
難度可選故事／標準／挑戰；事件包可篩選民主化、兩岸、地方民生或媒體信任議題；--timeline date 會將有日期錨點的事件周年映射到 12 週戰役日曆，沒有錨點的週次留白。這是可切換的遊戲排程，不表示史實在遊戲年份重演。
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
}
