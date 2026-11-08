use project_twn::history::{
    find_party, find_region, ELECTIONS, EVENTS, LEGISLATURE_SNAPSHOTS, PARTY_PROFILES, REGIONS,
    VOTER_SEGMENTS,
};
use project_twn::primary::{PrimaryChoice, PrimaryContest, PRIMARY_ROUNDS};
use project_twn::sim::{
    actions_for, ActionId, CampaignStyle, ElectionOutcome, GameState, Side, CAMPAIGN_ROUNDS,
    DAYS_PER_WEEK,
};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

const CAMPAIGN_LOG_HEADER: &str = "year\tmode\tregion\trole\tstyle\tseed\twinner\tplayer_win_probability\tobjectives_completed\tpolicy\ttrust\tmobilization\tevent_response\tresource_management\tversatility";

struct Options {
    year: u16,
    role: Option<Side>,
    style: CampaignStyle,
    seed: u64,
    auto: bool,
    history: bool,
    list_regions: bool,
    list_parties: bool,
    region: Option<String>,
    party: Option<String>,
    primary: bool,
    log: Option<String>,
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
    let mut game = match region {
        Some(region) => GameState::new_local_with_party(
            options.year,
            role,
            options.seed,
            style,
            region,
            selected_party,
        ),
        None => GameState::new_with_style(options.year, role, options.seed, style),
    };
    print_intro(&game, options.seed);

    if options.auto {
        run_auto(&mut game);
    } else {
        run_interactive(&mut game)?;
    }
    let result = print_final(&mut game);
    if let Some(path) = options.log.as_deref() {
        save_campaign_log(path, &game, &result, options.seed)?;
        println!("本場紀錄已追加至：{path}");
    }
    Ok(())
}

fn parse_args() -> Result<Options, String> {
    let mut args = env::args().skip(1);
    let mut options = Options {
        year: 2026,
        role: None,
        style: CampaignStyle::Balanced,
        seed: 2026,
        auto: false,
        history: false,
        list_regions: false,
        list_parties: false,
        region: None,
        party: None,
        primary: false,
        log: None,
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
            "--log" => {
                options.log = Some(
                    args.next()
                        .ok_or_else(|| "--log 後面需要 TSV 檔案路徑".to_string())?,
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

fn print_intro(game: &GameState, seed: u64) {
    println!("\n=== 臺灣選戰模擬器 | Project-TWN ===");
    println!(
        "情境年份：{}　陣營：{}　風格：{}　隨機種子：{}",
        game.campaign_year,
        game.side.label(),
        game.style.label(),
        seed
    );
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
            "歷史基準：{} 年總統選舉，初始民調以當年全國得票率為基準。",
            game.baseline_year
        );
    }
    if game.region.is_none() && game.campaign_year != game.baseline_year {
        println!("注意：這是以最近一次總統票作起點的替代情境，不是該年度的實際選舉結果。");
    }
    if game.campaign_year == 2026 {
        if game.region.is_some() {
            println!("2026地方選舉模式：2022年各地上屆首長政黨作為基準情境；2026投票日為 11-28，截至 11-08 尚無開票結果。" );
        } else {
            println!("2026全國代理情境：以2024總統票作全國基準，不是地方候選人或縣市民調；地方投票日為 2026-11-28。" );
        }
    }
    println!("競選期：{CAMPAIGN_ROUNDS} 週 × {DAYS_PER_WEEK} 天，共 {} 個每日決策（布局／議題攻防／決戰三階段）；起始預算：{}　信任：{:.1}/100　體力：{:.1}/100", CAMPAIGN_ROUNDS as u16 * DAYS_PER_WEEK as u16, game.budget, game.trust, game.stamina);
    println!(
        "每週會抽取一項陣營任務；達成可獲得最多 +1 預算。目標是遊戲任務，不代表現實政治建議。\n"
    );
    println!(
        "選民結構：{} 個遊戲用選民原型；權重是可調模型參數，不代表實際人口或選民占比。",
        VOTER_SEGMENTS.len()
    );
    println!(
        "玩家能力值（遊戲建構，非歷史評分）：政策 {:.0}／組織 {:.0}／溝通 {:.0}\n",
        game.player_attributes.policy,
        game.player_attributes.organization,
        game.player_attributes.communication
    );

    if !game.events.is_empty() {
        println!("本情境公共議題（事件本身不預設政黨得失；回應方式才改變模擬民調）：");
        for event in &game.events {
            println!("  • {}：{}", event.year, event.title);
        }
        println!();
    }
    print_poll_table(game);
    println!("\n行動的民調效果會受信任、議題契合、固定 seed 的小幅波動，以及對手回應影響。\n");
}

fn print_poll_table(game: &GameState) {
    println!("對比式：多候選人支持度（單一選票情境）");
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
    println!("選民原型細分（權重／玩家支持）— 僅供觀察遊戲模型：");
    for segment in &game.voter_segments {
        let player_support = segment.polls.get(game.player_index).copied().unwrap_or(0.0);
        println!(
            "  {:<20} {:>4.0}%／{:>5.1}%",
            segment.name, segment.weight_pct, player_support
        );
    }
    if game.candidates.len() > 2 {
        println!("互比式情境：若只比較玩家與單一對手，其他候選人的支持暫按各半轉移；這是遊戲簡化值，並非交叉民調。" );
        for (index, candidate) in game
            .candidates
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != game.player_index)
        {
            let undecided = (100.0 - game.polls[game.player_index] - game.polls[index]).max(0.0);
            let player_share = game.polls[game.player_index] + undecided * 0.5;
            println!(
                "  玩家 {:.1}% : {} {:.1}%",
                player_share,
                candidate.party,
                100.0 - player_share
            );
        }
    }
}

fn run_interactive(game: &mut GameState) -> Result<(), String> {
    while !game.is_finished() {
        println!(
            "\n--- 第 {} / {} 週・第 {} / {} 天｜{} ---",
            game.round + 1,
            CAMPAIGN_ROUNDS,
            game.day + 1,
            DAYS_PER_WEEK,
            game.phase_label()
        );
        println!(
            "剩餘預算：{}　信任：{:.1}　動員：{:.1}　體力：{:.1}",
            game.budget, game.trust, game.turnout_momentum, game.stamina
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
        let actions: Vec<_> = actions_for(game.side)
            .iter()
            .copied()
            .filter(|action| action_available(game, action.id))
            .collect();
        for (index, action) in actions.iter().enumerate() {
            println!(
                "  {}) {}（費用 {}）— {}",
                index + 1,
                action.name,
                action.cost,
                action.description
            );
        }
        print!("選行動（輸入編號，q 結束）：");
        io::stdout().flush().map_err(|error| error.to_string())?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        if input.trim().eq_ignore_ascii_case("q") {
            break;
        }
        let choice: usize = match input.trim().parse() {
            Ok(value) => value,
            Err(_) => {
                println!("請輸入行動編號。\n");
                continue;
            }
        };
        let Some(action) = actions.get(choice.saturating_sub(1)) else {
            println!("沒有這個選項。\n");
            continue;
        };
        match game.apply_action(action.id) {
            Ok(outcome) => print_action_outcome(game, &outcome),
            Err(message) => println!("{message}\n"),
        }
    }
    Ok(())
}

fn run_auto(game: &mut GameState) {
    let actions = actions_for(game.side);
    let preferred = match game.side {
        Side::Incumbent => [
            ActionId::PolicyDelivery,
            ActionId::CrisisResponse,
            ActionId::TransparencyReport,
            ActionId::PublicServices,
            ActionId::PollCoordination,
        ],
        Side::Challenger => [
            ActionId::CommunityTour,
            ActionId::PolicyForum,
            ActionId::LegalAssembly,
            ActionId::CoalitionTalks,
            ActionId::PollCoordination,
        ],
    };
    while !game.is_finished() {
        let preferred_id = preferred
            [(game.round as usize * DAYS_PER_WEEK as usize + game.day as usize) % preferred.len()];
        let objective_action = game
            .current_objective()
            .filter(|objective| game.current_objective_progress() < objective.target_actions)
            .and_then(|objective| {
                objective.qualifying_actions.iter().find(|id| {
                    actions.iter().any(|action| {
                        action.id == **id
                            && action.cost <= game.budget
                            && action_available(game, action.id)
                    })
                })
            })
            .copied();
        let selected = objective_action
            .and_then(|id| actions.iter().find(|action| action.id == id))
            .or_else(|| {
                actions.iter().find(|action| {
                    action.id == preferred_id
                        && action.cost <= game.budget
                        && action_available(game, action.id)
                })
            })
            .or_else(|| {
                actions
                    .iter()
                    .find(|action| action.cost <= game.budget && action_available(game, action.id))
            });
        let Some(action) = selected else { break };
        if let Ok(outcome) = game.apply_action(action.id) {
            print_action_outcome(game, &outcome);
        }
    }
}

fn print_action_outcome(game: &GameState, outcome: &project_twn::sim::ActionOutcome) {
    println!(
        "✓ 第 {} 週・第 {} 天 {}：玩家民調 {:.2}% → {:.2}%（費用 {}、本週補給 +{}、餘額 {}）",
        outcome.week,
        outcome.day,
        outcome.action_name,
        outcome.share_before,
        outcome.share_after,
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
        "  信任 {:.1}/100，動員指數 {:.1}/100，體力 {:.1}/100",
        outcome.trust_after, outcome.turnout_after, outcome.stamina_after
    );
    if outcome.day == DAYS_PER_WEEK {
        print_poll_table(game);
    }
}

fn action_available(game: &GameState, id: ActionId) -> bool {
    match id {
        ActionId::PollCoordination => game.candidates.len() >= 3 && !game.poll_taken_this_week,
        ActionId::CoalitionTalks => game.candidates.len() >= 3 && !game.coalition_this_week,
        _ => true,
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
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.4}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}",
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
        seed,
        winner.party,
        result.player_win_probability,
        game.objectives_completed(),
        review.policy,
        review.trust,
        review.mobilization,
        review.event_response,
        review.resource_management,
        review.versatility
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
    println!("Project-TWN — 臺灣選戰模擬器\n\n使用方式：\n  project-twn --history\n  project-twn --regions\n  project-twn --parties\n  project-twn --simulate [年份] [--region 縣市] [--party 政黨] [--role incumbent|challenger] [--style balanced|policy|organizer|communicator] [--primary] [--seed 整數] [--auto] [--log 檔案.tsv]\n\n範例：\n  cargo run -- --simulate 2024 --role challenger --style organizer --seed 42\n  cargo run -- --year 2026 --region 臺北市 --role challenger --party 時代力量 --seed 7 --auto\n  cargo run -- --history\n  cargo run -- --regions\n  cargo run -- --parties\n\n年份範圍：1996–2026。地方模式可用 --regions 檢視 22 個縣市；--parties 檢視歷史政黨範本。競選為 12 週、每週 7 天，共 84 次每日行動決策；每週有資金補給。--primary 先進行四回合黨內初選。\n省略 --role 會詢問陣營；--auto 會以在野方跑完示範。--log 可將每場結果與戰役操作評分附加至 TSV，累積長期戰績。");
}
