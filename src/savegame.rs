//! Versioned replay-journal persistence for deterministic campaign sessions.
//! The journal stores player decisions and replays them from the original seed.

use crate::roster::{CandidateRoster, RosterEntry};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const HEADER: &str = "PROJECT_TWN_SAVE\t1";
const MAX_SAVE_BYTES: u64 = 1_000_000;
const MAX_COMMANDS: usize = 2_000;

#[derive(Clone, Debug, PartialEq)]
pub struct CampaignSaveConfig {
    pub year: u16,
    pub side: String,
    pub style: String,
    pub strategy: String,
    pub seed: u64,
    pub region: Option<String>,
    pub party: Option<String>,
    pub difficulty: String,
    pub event_pack: String,
    pub timeline: String,
    pub staffer: bool,
    pub roster: Option<CandidateRoster>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecallSaveConfig {
    pub side: String,
    pub template: String,
    pub seed: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SaveConfig {
    Campaign(CampaignSaveConfig),
    Recall(RecallSaveConfig),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveCommand {
    Action(usize),
    StoryChoice(usize),
    SkipStory,
    StafferChoice(usize),
    SkipStaffer,
    RecallAction(usize),
    SettleEarly,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SaveGame {
    pub config: SaveConfig,
    pub commands: Vec<SaveCommand>,
}

impl SaveGame {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).map_err(|error| format!("無法讀取存檔資訊：{error}"))?;
        if metadata.len() > MAX_SAVE_BYTES {
            return Err("存檔超過 1 MB 安全上限。".to_string());
        }
        let contents =
            fs::read_to_string(path).map_err(|error| format!("無法讀取存檔：{error}"))?;
        let mut lines = contents.lines();
        if lines.next() != Some(HEADER) {
            return Err("存檔版本不支援；此版本僅能讀取 PROJECT_TWN_SAVE 1。".to_string());
        }
        let mut values = HashMap::new();
        let mut candidates = Vec::new();
        let mut commands = Vec::new();
        for (line_index, line) in lines.enumerate() {
            if line.is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split('\t').collect();
            match fields.first().copied() {
                Some("cfg") if fields.len() == 3 => {
                    let key = decode(fields[1])?;
                    let value = decode(fields[2])?;
                    if values.insert(key.clone(), value).is_some() {
                        return Err(format!("存檔設定重複：{key}"));
                    }
                }
                Some("candidate") if fields.len() == 5 => {
                    if candidates.len() >= 5 {
                        return Err("存檔候選人數超出上限。".to_string());
                    }
                    let name = decode(fields[1])?;
                    let party = decode(fields[2])?;
                    let starting_support = fields[3].parse::<f64>().map_err(|_| {
                        format!("存檔第 {} 行候選人支持度格式錯誤。", line_index + 2)
                    })?;
                    let player = match fields[4] {
                        "player" => true,
                        "opponent" => false,
                        _ => {
                            return Err(format!("存檔第 {} 行候選人角色格式錯誤。", line_index + 2))
                        }
                    };
                    candidates.push(RosterEntry {
                        name,
                        party,
                        starting_support,
                        player,
                    });
                }
                Some("command") if fields.len() == 3 => {
                    if commands.len() >= MAX_COMMANDS {
                        return Err("存檔決策數量超出上限。".to_string());
                    }
                    let index = fields[2]
                        .parse::<usize>()
                        .map_err(|_| format!("存檔第 {} 行決策索引錯誤。", line_index + 2))?;
                    let command = match fields[1] {
                        "action" if index < 30 => SaveCommand::Action(index),
                        "story" if index < 4 => SaveCommand::StoryChoice(index),
                        "skip_story" if index == 0 => SaveCommand::SkipStory,
                        "staffer" if index < 4 => SaveCommand::StafferChoice(index),
                        "skip_staffer" if index == 0 => SaveCommand::SkipStaffer,
                        "recall" if index < 14 => SaveCommand::RecallAction(index),
                        "settle_early" if index == 0 => SaveCommand::SettleEarly,
                        _ => {
                            return Err(format!(
                                "存檔第 {} 行決策種類或索引不支援。",
                                line_index + 2
                            ))
                        }
                    };
                    commands.push(command);
                }
                _ => return Err(format!("存檔第 {} 行欄位不正確。", line_index + 2)),
            }
        }
        let mode = take(&mut values, "mode")?;
        let config = match mode.as_str() {
            "campaign" => {
                let year = parse_value::<u16>(&mut values, "year")?;
                if !(1996..=2026).contains(&year) {
                    return Err("存檔戰役年份超出可用範圍。".to_string());
                }
                let side = take(&mut values, "side")?;
                if !["incumbent", "challenger"].contains(&side.as_str()) {
                    return Err("存檔競選陣營格式錯誤。".to_string());
                }
                let style = take(&mut values, "style")?;
                let strategy = take(&mut values, "strategy")?;
                let seed = parse_value::<u64>(&mut values, "seed")?;
                let region = take_optional(&mut values, "region")?;
                let party = take_optional(&mut values, "party")?;
                let difficulty = take(&mut values, "difficulty")?;
                let event_pack = take(&mut values, "event_pack")?;
                let timeline = take(&mut values, "timeline")?;
                let staffer = match take(&mut values, "staffer")?.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err("存檔幕僚模式旗標錯誤。".to_string()),
                };
                let roster = if candidates.is_empty() {
                    None
                } else {
                    let roster = CandidateRoster { candidates };
                    roster.validate()?;
                    Some(roster)
                };
                SaveConfig::Campaign(CampaignSaveConfig {
                    year,
                    side,
                    style,
                    strategy,
                    seed,
                    region,
                    party,
                    difficulty,
                    event_pack,
                    timeline,
                    staffer,
                    roster,
                })
            }
            "recall" => {
                if !candidates.is_empty() {
                    return Err("罷免存檔不得包含候選人名冊。".to_string());
                }
                if commands
                    .iter()
                    .any(|command| !matches!(command, SaveCommand::RecallAction(_)))
                {
                    return Err("罷免存檔含有不相容的競選決策。".to_string());
                }
                let side = take(&mut values, "side")?;
                if !["initiator", "opposition"].contains(&side.as_str()) {
                    return Err("存檔罷免陣營格式錯誤。".to_string());
                }
                let template = take(&mut values, "template")?;
                let seed = parse_value::<u64>(&mut values, "seed")?;
                SaveConfig::Recall(RecallSaveConfig {
                    side,
                    template,
                    seed,
                })
            }
            _ => return Err("存檔遊戲模式不支援。".to_string()),
        };
        if !values.is_empty() {
            return Err(format!(
                "存檔包含未支援的設定欄位：{}",
                values.keys().next().unwrap()
            ));
        }
        match &config {
            SaveConfig::Campaign(_)
                if commands
                    .iter()
                    .any(|command| matches!(command, SaveCommand::RecallAction(_))) =>
            {
                return Err("競選存檔含有不相容的罷免決策。".to_string());
            }
            SaveConfig::Recall(_)
                if commands
                    .iter()
                    .any(|command| !matches!(command, SaveCommand::RecallAction(_))) =>
            {
                return Err("罷免存檔含有不相容的競選決策。".to_string());
            }
            _ => {}
        }
        Ok(Self { config, commands })
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), String> {
        if self.commands.len() > MAX_COMMANDS {
            return Err("存檔決策數量超出上限。".to_string());
        }
        let path = path.as_ref();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|error| format!("無法建立存檔資料夾：{error}"))?;
        }
        let temp = path.with_extension(format!("save-{}.tmp", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| format!("無法建立存檔暫存檔：{error}"))?;
        let result = write_body(&mut file, self).and_then(|()| file.sync_all());
        if let Err(error) = result {
            let _ = fs::remove_file(&temp);
            return Err(format!("無法寫入存檔：{error}"));
        }
        fs::rename(&temp, path).map_err(|error| {
            let _ = fs::remove_file(&temp);
            format!("無法以新版存檔取代目標檔案：{error}")
        })
    }
}

fn write_body(file: &mut impl Write, save: &SaveGame) -> std::io::Result<()> {
    let mut body = Vec::new();
    writeln!(&mut body, "{HEADER}")?;
    let mut config = Vec::<(&str, String)>::new();
    match &save.config {
        SaveConfig::Campaign(campaign) => {
            config.extend([
                ("mode", "campaign".to_string()),
                ("year", campaign.year.to_string()),
                ("side", campaign.side.clone()),
                ("style", campaign.style.clone()),
                ("strategy", campaign.strategy.clone()),
                ("seed", campaign.seed.to_string()),
                ("region", campaign.region.clone().unwrap_or_default()),
                ("party", campaign.party.clone().unwrap_or_default()),
                ("difficulty", campaign.difficulty.clone()),
                ("event_pack", campaign.event_pack.clone()),
                ("timeline", campaign.timeline.clone()),
                (
                    "staffer",
                    if campaign.staffer { "1" } else { "0" }.to_string(),
                ),
            ]);
        }
        SaveConfig::Recall(recall) => config.extend([
            ("mode", "recall".to_string()),
            ("side", recall.side.clone()),
            ("template", recall.template.clone()),
            ("seed", recall.seed.to_string()),
        ]),
    }
    for (key, value) in config {
        writeln!(&mut body, "cfg\t{}\t{}", encode(key), encode(&value))?;
    }
    if let SaveConfig::Campaign(campaign) = &save.config {
        if let Some(roster) = &campaign.roster {
            for candidate in &roster.candidates {
                writeln!(
                    &mut body,
                    "candidate\t{}\t{}\t{:.6}\t{}",
                    encode(&candidate.name),
                    encode(&candidate.party),
                    candidate.starting_support,
                    if candidate.player {
                        "player"
                    } else {
                        "opponent"
                    }
                )?;
            }
        }
    }
    for command in &save.commands {
        let (kind, index) = match command {
            SaveCommand::Action(index) => ("action", *index),
            SaveCommand::StoryChoice(index) => ("story", *index),
            SaveCommand::SkipStory => ("skip_story", 0),
            SaveCommand::StafferChoice(index) => ("staffer", *index),
            SaveCommand::SkipStaffer => ("skip_staffer", 0),
            SaveCommand::RecallAction(index) => ("recall", *index),
            SaveCommand::SettleEarly => ("settle_early", 0),
        };
        writeln!(&mut body, "command\t{kind}\t{index}")?;
    }
    file.write_all(&body)
}

fn take(values: &mut HashMap<String, String>, key: &str) -> Result<String, String> {
    values
        .remove(key)
        .ok_or_else(|| format!("存檔缺少必要欄位：{key}"))
}

fn take_optional(
    values: &mut HashMap<String, String>,
    key: &str,
) -> Result<Option<String>, String> {
    let value = take(values, key)?;
    Ok((!value.is_empty()).then_some(value))
}

fn parse_value<T: std::str::FromStr>(
    values: &mut HashMap<String, String>,
    key: &str,
) -> Result<T, String> {
    take(values, key)?
        .parse::<T>()
        .map_err(|_| format!("存檔欄位格式錯誤：{key}"))
}

fn encode(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for byte in value.bytes() {
        if !byte.is_ascii_alphanumeric() && !matches!(byte, b' ' | b'-' | b'_' | b'.') {
            result.push_str(&format!("%{byte:02X}"));
        } else {
            result.push(byte as char);
        }
    }
    result
}

fn decode(value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let pair = bytes
                .get(index + 1..index + 3)
                .ok_or_else(|| "存檔跳脫字元格式錯誤。".to_string())?;
            let hex =
                std::str::from_utf8(pair).map_err(|_| "存檔跳脫字元格式錯誤。".to_string())?;
            decoded.push(
                u8::from_str_radix(hex, 16).map_err(|_| "存檔跳脫字元格式錯誤。".to_string())?,
            );
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).map_err(|_| "存檔文字不是有效 UTF-8。".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SaveGame {
        SaveGame {
            config: SaveConfig::Campaign(CampaignSaveConfig {
                year: 2024,
                side: "challenger".to_string(),
                style: "balanced".to_string(),
                strategy: "issue".to_string(),
                seed: 7,
                region: None,
                party: None,
                difficulty: "standard".to_string(),
                event_pack: "all".to_string(),
                timeline: "weekly".to_string(),
                staffer: false,
                roster: Some(CandidateRoster {
                    candidates: vec![
                        RosterEntry {
                            name: "王%小明".to_string(),
                            party: "群島共治黨".to_string(),
                            starting_support: 48.0,
                            player: true,
                        },
                        RosterEntry {
                            name: "林沐青".to_string(),
                            party: "曙光民主聯盟".to_string(),
                            starting_support: 52.0,
                            player: false,
                        },
                    ],
                }),
            }),
            commands: vec![SaveCommand::StoryChoice(2), SaveCommand::Action(3)],
        }
    }

    #[test]
    fn save_file_round_trips_config_roster_and_action_journal() {
        let path =
            std::env::temp_dir().join(format!("project-twn-save-{}.tsv", std::process::id()));
        let value = sample();
        value.write(&path).unwrap();
        assert_eq!(SaveGame::read(&path).unwrap(), value);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn save_reader_rejects_unknown_version_and_mixed_command_types() {
        let path = std::env::temp_dir().join(format!(
            "project-twn-invalid-save-{}.tsv",
            std::process::id()
        ));
        std::fs::write(&path, "PROJECT_TWN_SAVE\t2\n").unwrap();
        assert!(SaveGame::read(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
