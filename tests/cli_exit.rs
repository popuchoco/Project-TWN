use project_twn::savegame::{SaveCommand, SaveGame};
use std::io::Write;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn unique_path(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "project-twn-{label}-{}-{nonce}.tsv",
        std::process::id()
    ))
}

fn spawn_cli(args: &[&str], input: &[u8]) -> Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_project-twn"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI executable should start");
    child
        .stdin
        .take()
        .expect("stdin should be piped")
        .write_all(input)
        .expect("input should be delivered");
    child
}

fn finish_with_timeout(mut child: Child) -> Output {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if child
            .try_wait()
            .expect("child should be waitable")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let output = child
                .wait_with_output()
                .expect("timed-out child output should be readable");
            panic!(
                "CLI failed to stop on EOF; captured {} stdout bytes and {} stderr bytes",
                output.stdout.len(),
                output.stderr.len()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    child
        .wait_with_output()
        .expect("child output should be readable")
}

fn run_cli(args: &[&str], input: &[u8]) -> Output {
    finish_with_timeout(spawn_cli(args, input))
}

#[test]
fn eof_after_mode_choice_exits_instead_of_spinning_at_role_prompt() {
    let output = run_cli(&["--seed", "1"], b"1\n");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("執政／在野"));
    assert!(output.stdout.len() < 32_768);
}

#[test]
fn eof_in_recall_and_primary_prompts_exits_cleanly() {
    let recall = run_cli(&["--recall", "--seed", "1"], b"");
    assert!(recall.status.success());
    assert!(recall.stdout.len() < 32_768);

    let primary = run_cli(&["--primary", "--role", "challenger", "--seed", "1"], b"");
    assert!(primary.status.success());
    assert!(String::from_utf8_lossy(&primary.stdout).contains("黨內初選"));
    assert!(primary.stdout.len() < 32_768);
}

#[test]
fn primary_accepts_q_as_a_clean_exit() {
    let output = run_cli(
        &["--primary", "--role", "challenger", "--seed", "1"],
        b"q\n",
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("q 離開初選"));
    assert!(!stdout.contains("初選提名："));
}

#[test]
fn eof_with_save_keeps_campaign_open_and_resumable() {
    let path = unique_path("eof-save");
    let path_text = path.to_string_lossy().to_string();
    let output = run_cli(
        &["--role", "challenger", "--seed", "7", "--save", &path_text],
        b"",
    );

    assert!(output.status.success());
    let save = SaveGame::read(&path).expect("EOF should leave a readable checkpoint");
    assert!(!save
        .commands
        .iter()
        .any(|command| matches!(command, SaveCommand::SettleEarly)));
    assert!(String::from_utf8_lossy(&output.stdout).contains("可用 `--load"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn save_failures_warn_and_do_not_abort_the_session() {
    let path = unique_path("blocked-save");
    std::fs::create_dir_all(&path).unwrap();
    let path_text = path.to_string_lossy().to_string();
    let output = run_cli(
        &["--role", "challenger", "--seed", "7", "--save", &path_text],
        b"",
    );

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("存檔警告"));
    assert!(String::from_utf8_lossy(&output.stdout).contains("存檔重試仍失敗"));
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn zero_returns_from_action_menus_without_executing_first_action() {
    for input in [b"0\nq\ny\n".as_slice(), b"1\n0\nq\ny\n".as_slice()] {
        let path = unique_path("zero-menu");
        let path_text = path.to_string_lossy().to_string();
        let output = run_cli(
            &["--role", "challenger", "--seed", "7", "--save", &path_text],
            input,
        );

        assert!(output.status.success());
        let save = SaveGame::read(&path).expect("save should remain readable");
        assert!(
            save.commands.is_empty(),
            "zero must not execute or journal an action"
        );
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn loading_and_leaving_without_new_decisions_does_not_rewrite_the_save() {
    let path = unique_path("load-preserve");
    let save = SaveGame {
        config: project_twn::savegame::SaveConfig::Campaign(
            project_twn::savegame::CampaignSaveConfig {
                year: 2024,
                side: "challenger".to_string(),
                style: "balanced".to_string(),
                strategy: "balanced".to_string(),
                seed: 77,
                region: None,
                party: None,
                difficulty: "standard".to_string(),
                event_pack: "all".to_string(),
                timeline: "weekly".to_string(),
                staffer: false,
                roster: None,
            },
        ),
        commands: Vec::new(),
    };
    save.write(&path).unwrap();
    let contents = std::fs::read(&path).unwrap();
    let v1 = String::from_utf8(contents).unwrap().replacen(
        "PROJECT_TWN_SAVE\t2",
        "PROJECT_TWN_SAVE\t1",
        1,
    );
    std::fs::write(&path, v1.as_bytes()).unwrap();
    let before = std::fs::read(&path).unwrap();
    let path_text = path.to_string_lossy().to_string();

    let output = run_cli(&["--load", &path_text], b"");

    assert!(output.status.success());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn broken_stdout_pipe_exits_without_panic_log() {
    let log_path = unique_path("broken-pipe-log").with_extension("log");
    let mut child = Command::new(env!("CARGO_BIN_EXE_project-twn"))
        .env("PROJECT_TWN_CRASH_LOG", &log_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("CLI executable should start");
    drop(child.stdout.take());

    let output = finish_with_timeout(child);

    assert!(output.status.success());
    assert!(!log_path.exists(), "broken pipes are not game crashes");
}

#[test]
fn cli_errors_are_concise_and_write_diagnostics() {
    let log_path = unique_path("cli-error-log").with_extension("log");
    let output = Command::new(env!("CARGO_BIN_EXE_project-twn"))
        .arg("--not-a-real-option")
        .env("PROJECT_TWN_CRASH_LOG", &log_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("CLI executable should start");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("使用 --help"));
    assert!(!stderr.contains("使用方式："));
    let diagnostics = std::fs::read_to_string(&log_path).unwrap();
    assert!(diagnostics.contains("Project-TWN error"));
    assert!(diagnostics.contains("--not-a-real-option"));
    std::fs::remove_file(log_path).unwrap();
}

#[test]
fn version_reports_build_version_and_commit() {
    let output = run_cli(&["--version"], b"");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("Project-TWN "));
    assert!(stdout.contains('(') && stdout.contains(')'));
}

#[test]
fn roster_editor_reprompts_only_for_duplicate_name() {
    let path = unique_path("roster-editor");
    let path_text = path.to_string_lossy().to_string();
    let output = run_cli(
        &["--roster-editor", &path_text],
        "2\n甲角\n群島共治黨\n0\ny\n甲角\n乙角\n曙光民主聯盟\n100\nn\n".as_bytes(),
    );

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("姓名重複"));
    let roster = project_twn::roster::CandidateRoster::read(&path)
        .expect("correcting one field should finish roster creation");
    assert_eq!(roster.candidates.len(), 2);
    assert_eq!(roster.candidates[1].name, "乙角");
    std::fs::remove_file(path).unwrap();
}
