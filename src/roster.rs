//! Editable, fictional candidate rosters for local play.

use crate::file_ops::replace_file;
use crate::game_content::find_party;
use crate::sim::GameState;
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

const HEADER: &str = "PROJECT_TWN_ROSTER\t1";

#[derive(Clone, Debug, PartialEq)]
pub struct RosterEntry {
    pub name: String,
    pub party: String,
    pub starting_support: f64,
    pub player: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CandidateRoster {
    pub candidates: Vec<RosterEntry>,
}

impl CandidateRoster {
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=5).contains(&self.candidates.len()) {
            return Err("候選人名冊需有 2 至 5 位候選人。".to_string());
        }
        let mut names = HashSet::new();
        let mut players = 0;
        let mut total = 0.0;
        for candidate in &self.candidates {
            let name = candidate.name.trim();
            if name.chars().count() < 2
                || name.chars().count() > 24
                || name.chars().any(|character| {
                    character.is_control() || matches!(character, '\t' | '\n' | '\r')
                })
            {
                return Err("候選人姓名需為 2 至 24 字，且不可含控制字元或欄位分隔符。".to_string());
            }
            if !names.insert(name.to_string()) {
                return Err(format!("候選人姓名重複：{name}"));
            }
            if find_party(&candidate.party).is_none() {
                return Err(format!(
                    "政黨不在虛構名冊：{}。請使用 --parties 查看清單。",
                    candidate.party
                ));
            }
            if !candidate.starting_support.is_finite()
                || !(0.0..=100.0).contains(&candidate.starting_support)
            {
                return Err(format!("{} 的起始支持度需介於 0 至 100。", candidate.name));
            }
            if candidate.player {
                players += 1;
            }
            total += candidate.starting_support;
        }
        if players != 1 {
            return Err("名冊必須且只能指定一位玩家候選人。".to_string());
        }
        if total <= 0.0 {
            return Err("候選人起始支持度總和必須大於 0。".to_string());
        }
        Ok(())
    }

    pub fn read(path: impl AsRef<Path>) -> Result<Self, String> {
        let contents = fs::read_to_string(path.as_ref())
            .map_err(|error| format!("無法讀取候選人名冊：{error}"))?;
        let mut lines = contents.lines();
        if lines
            .next()
            .map(|header| header.strip_prefix('\u{feff}').unwrap_or(header))
            != Some(HEADER)
        {
            return Err(
                "候選人名冊版本不支援；檔案首行須為 PROJECT_TWN_ROSTER<TAB>1。".to_string(),
            );
        }
        if lines.next() != Some("name\tparty\tstarting_support\tplayer") {
            return Err("候選人名冊欄位不正確。".to_string());
        }
        let mut candidates = Vec::new();
        for (line_number, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let fields: Vec<_> = line.split('\t').collect();
            if fields.len() != 4 {
                return Err(format!("候選人名冊第 {} 行欄位數錯誤。", line_number + 3));
            }
            let starting_support = fields[2]
                .parse::<f64>()
                .map_err(|_| format!("候選人名冊第 {} 行支持度格式錯誤。", line_number + 3))?;
            let player = match fields[3] {
                "player" => true,
                "opponent" => false,
                _ => {
                    return Err(format!(
                        "候選人名冊第 {} 行陣營需為 player 或 opponent。",
                        line_number + 3
                    ))
                }
            };
            candidates.push(RosterEntry {
                name: fields[0].to_string(),
                party: fields[1].to_string(),
                starting_support,
                player,
            });
        }
        let roster = Self { candidates };
        roster.validate()?;
        Ok(roster)
    }

    pub fn write(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;
        let path = path.as_ref();
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent).map_err(|error| format!("無法建立名冊資料夾：{error}"))?;
        let temp = path.with_extension(format!("roster-{}.tmp", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| format!("無法建立名冊暫存檔：{error}"))?;
        let write_result = (|| -> std::io::Result<()> {
            writeln!(file, "{HEADER}")?;
            writeln!(file, "name\tparty\tstarting_support\tplayer")?;
            for candidate in &self.candidates {
                writeln!(
                    file,
                    "{}\t{}\t{}\t{}",
                    candidate.name,
                    candidate.party,
                    candidate.starting_support,
                    if candidate.player {
                        "player"
                    } else {
                        "opponent"
                    }
                )?;
            }
            file.sync_all()
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&temp);
            return Err(format!("無法寫入候選人名冊：{error}"));
        }
        replace_file(&temp, path).map_err(|error| {
            let _ = fs::remove_file(&temp);
            format!("無法以新版名冊取代目標檔案：{error}")
        })
    }

    pub fn apply(&self, game: &mut GameState) -> Result<(), String> {
        self.validate()?;
        let entries = self
            .candidates
            .iter()
            .map(|candidate| {
                let party = find_party(&candidate.party).expect("validated fictional party");
                (
                    candidate.name.clone(),
                    party.name,
                    candidate.starting_support,
                )
            })
            .collect::<Vec<_>>();
        let player_index = self
            .candidates
            .iter()
            .position(|candidate| candidate.player)
            .expect("validated player candidate");
        game.apply_candidate_roster(&entries, player_index)
            .map_err(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::{CampaignStyle, GameState, Side};

    fn roster() -> CandidateRoster {
        CandidateRoster {
            candidates: vec![
                RosterEntry {
                    name: "沈知行".to_string(),
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
        }
    }

    #[test]
    fn versioned_roster_round_trips() {
        let path =
            std::env::temp_dir().join(format!("project-twn-roster-{}.tsv", std::process::id()));
        let roster = roster();
        roster.write(&path).unwrap();
        assert_eq!(CandidateRoster::read(&path).unwrap(), roster);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn roster_requires_a_single_player_and_known_fictional_parties() {
        let mut value = roster();
        value.candidates[1].player = true;
        assert!(value.validate().is_err());
        value.candidates[1].player = false;
        value.candidates[0].party = "未知政黨".to_string();
        assert!(value.validate().is_err());
    }

    #[test]
    fn roster_applies_names_parties_support_and_player_assignment_before_play() {
        let roster = roster();
        let mut game =
            GameState::new_with_style(2024, Side::Challenger, 8, CampaignStyle::Balanced);
        roster.apply(&mut game).unwrap();
        assert_eq!(game.candidates.len(), 2);
        assert_eq!(game.candidates[game.player_index].name, "沈知行");
        assert_eq!(game.candidates[game.player_index].party, "群島共治黨");
        assert!((game.polls.iter().sum::<f64>() - 100.0).abs() < 1e-9);
        assert!(game.validate().is_ok());
    }

    #[test]
    fn roster_preserves_small_nonzero_support_values() {
        let path = std::env::temp_dir().join(format!(
            "project-twn-roster-small-{}.tsv",
            std::process::id()
        ));
        let mut value = roster();
        value.candidates[0].starting_support = 0.000_01;
        value.candidates[1].starting_support = 99.999_99;

        value.write(&path).unwrap();

        assert_eq!(CandidateRoster::read(&path).unwrap(), value);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn roster_reader_accepts_utf8_bom() {
        let path =
            std::env::temp_dir().join(format!("project-twn-roster-bom-{}.tsv", std::process::id()));
        let value = roster();
        value.write(&path).unwrap();
        let contents = std::fs::read(&path).unwrap();
        let mut with_bom = b"\xef\xbb\xbf".to_vec();
        with_bom.extend(contents);
        std::fs::write(&path, with_bom).unwrap();

        assert_eq!(CandidateRoster::read(&path).unwrap(), value);
        std::fs::remove_file(path).unwrap();
    }
}
