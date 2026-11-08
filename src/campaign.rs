//! Per-campaign content and pacing options.
//!
//! These settings select playable content and difficulty; they are not
//! historical accuracy parameters or election forecasts.

use crate::history::{EventCard, Issue};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CampaignDifficulty {
    Story,
    #[default]
    Standard,
    Hard,
}

impl CampaignDifficulty {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "story" | "storymode" | "故事" | "輕鬆" => Some(Self::Story),
            "standard" | "normal" | "標準" => Some(Self::Standard),
            "hard" | "困難" | "挑戰" => Some(Self::Hard),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Story => "story",
            Self::Standard => "standard",
            Self::Hard => "hard",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Story => "故事難度",
            Self::Standard => "標準難度",
            Self::Hard => "挑戰難度",
        }
    }

    pub const fn starting_budget(self) -> i32 {
        match self {
            Self::Story => 110,
            Self::Standard => 100,
            Self::Hard => 90,
        }
    }

    pub const fn weekly_funding(self) -> i32 {
        match self {
            Self::Story => 10,
            Self::Standard => 8,
            Self::Hard => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventPack {
    #[default]
    All,
    Democracy,
    CrossStrait,
    LocalLife,
    MediaTrust,
}

impl EventPack {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "all" | "全部" | "綜合" => Some(Self::All),
            "democracy" | "民主" | "民主化" => Some(Self::Democracy),
            "cross-strait" | "crossstrait" | "兩岸" | "國際" => Some(Self::CrossStrait),
            "local" | "local-life" | "地方" | "民生" => Some(Self::LocalLife),
            "media" | "media-trust" | "媒體" | "信任" => Some(Self::MediaTrust),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Democracy => "democracy",
            Self::CrossStrait => "cross-strait",
            Self::LocalLife => "local",
            Self::MediaTrust => "media",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "綜合事件包",
            Self::Democracy => "民主化與公民運動",
            Self::CrossStrait => "兩岸與區域安全",
            Self::LocalLife => "地方治理與民生",
            Self::MediaTrust => "媒體與政治信任",
        }
    }

    pub fn includes(self, event: &EventCard) -> bool {
        match self {
            Self::All => true,
            Self::Democracy => matches!(
                event.issue,
                Issue::Civic | Issue::Education | Issue::Rights | Issue::Governance | Issue::Trust
            ),
            Self::CrossStrait => matches!(
                event.issue,
                Issue::CrossStrait | Issue::Civic | Issue::Rights
            ),
            Self::LocalLife => matches!(
                event.issue,
                Issue::Local | Issue::Economy | Issue::PublicHealth | Issue::Education
            ),
            Self::MediaTrust => matches!(event.issue, Issue::Media | Issue::Trust),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventTimeline {
    #[default]
    SeededWeekly,
    AnniversaryMapped,
}

impl EventTimeline {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "weekly" | "seeded" | "週次" | "隨機" => Some(Self::SeededWeekly),
            "date" | "anniversary" | "日期" | "周年" => Some(Self::AnniversaryMapped),
            _ => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::SeededWeekly => "weekly",
            Self::AnniversaryMapped => "date",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::SeededWeekly => "固定種子週次排程",
            Self::AnniversaryMapped => "事件周年日期映射",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CampaignSetup {
    pub difficulty: CampaignDifficulty,
    pub event_pack: EventPack,
    pub timeline: EventTimeline,
}

impl CampaignSetup {
    pub fn events(self, events: &[&'static EventCard]) -> Vec<&'static EventCard> {
        events
            .iter()
            .copied()
            .filter(|event| self.event_pack.includes(event))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DatedEvent {
    pub title: &'static str,
    pub date: &'static str,
}

/// Curated date anchors for optional anniversary mapping. The campaign clock
/// is a game calendar; mapping an anniversary to a week does not claim that
/// the historical event happened again in the scenario year.
pub const DATED_EVENTS: &[DatedEvent] = &[
    DatedEvent {
        title: "野百合學運與國會／憲政改革",
        date: "1990-03-16",
    },
    DatedEvent {
        title: "終止動員戡亂與憲政轉型",
        date: "1991-05-01",
    },
    DatedEvent {
        title: "首次總統直選",
        date: "1996-03-23",
    },
    DatedEvent {
        title: "三一九槍擊案與選後爭議",
        date: "2004-03-19",
    },
    DatedEvent {
        title: "洪仲丘事件與軍中人權改革",
        date: "2013-07-03",
    },
    DatedEvent {
        title: "太陽花學運",
        date: "2014-03-18",
    },
    DatedEvent {
        title: "周子瑜遭貼台獨標籤與道歉影片爭議",
        date: "2016-01-15",
    },
    DatedEvent {
        title: "同性婚姻法制化",
        date: "2019-05-24",
    },
    DatedEvent {
        title: "COVID-19疫情治理",
        date: "2020-01-21",
    },
    DatedEvent {
        title: "香港國安法實施",
        date: "2020-06-30",
    },
    DatedEvent {
        title: "香港基本法第23條立法",
        date: "2024-03-23",
    },
    DatedEvent {
        title: "政治檔案條例修正與轉型正義",
        date: "2024-02-28",
    },
    DatedEvent {
        title: "香港國安法第43條實施細則修訂",
        date: "2026-03-23",
    },
    DatedEvent {
        title: "地方公職人員選舉（尚未投票）",
        date: "2026-11-28",
    },
];

/// Map a historical event's month/day anniversary onto the twelve-week
/// campaign window immediately before an election date. Returns a zero-based
/// campaign week, or `None` when that anniversary falls outside the window.
pub fn anniversary_week(event_date: &str, election_date: &str) -> Option<u8> {
    anniversary_day_offset(event_date, election_date)
        .map(|days_before| 11 - (days_before / 7) as u8)
}

/// Return the number of days between the event anniversary and election day
/// when the date falls inside the twelve-week campaign window.
pub fn anniversary_day_offset(event_date: &str, election_date: &str) -> Option<u16> {
    let (event_month, event_day) = month_day(event_date)?;
    let (election_year, election_month, election_day) = full_date(election_date)?;
    let days_in_year = if is_leap_year(election_year) {
        366
    } else {
        365
    };
    let event_ordinal = ordinal(election_year, event_month, event_day)?;
    let election_ordinal = ordinal(election_year, election_month, election_day)?;
    let days_before = (election_ordinal + days_in_year - event_ordinal) % days_in_year;
    if days_before >= 12 * 7 {
        return None;
    }
    Some(days_before)
}

fn full_date(value: &str) -> Option<(u16, u8, u8)> {
    let mut parts = value.split('-');
    Some((
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
        parts.next()?.parse().ok()?,
    ))
}

fn month_day(value: &str) -> Option<(u8, u8)> {
    let (_, month, day) = full_date(value)?;
    Some((month, day))
}

fn is_leap_year(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn ordinal(year: u16, month: u8, day: u8) -> Option<u16> {
    let month_lengths = [
        31,
        if is_leap_year(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if month == 0 || month > 12 || day == 0 || day > month_lengths[usize::from(month - 1)] {
        return None;
    }
    Some(month_lengths[..usize::from(month - 1)].iter().sum::<u8>() as u16 + u16::from(day - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_date_anniversary_to_week_before_election() {
        assert_eq!(anniversary_week("2004-03-19", "2004-03-20"), Some(11));
        assert_eq!(anniversary_week("2016-01-15", "2016-01-16"), Some(11));
        assert_eq!(anniversary_week("2020-01-21", "2020-01-11"), None);
        assert_eq!(anniversary_week("2000-01-02", "2000-03-25"), Some(0));
    }

    #[test]
    fn rejects_invalid_date_and_event_packs_split_content() {
        assert_eq!(anniversary_week("2020-02-30", "2020-03-20"), None);
        assert_ne!(EventPack::Democracy, EventPack::All);
        assert_ne!(EventPack::LocalLife, EventPack::MediaTrust);
        let three_nineteen = crate::history::EVENTS
            .iter()
            .find(|event| event.title == "三一九槍擊案與選後爭議")
            .unwrap();
        assert!(EventPack::Democracy.includes(three_nineteen));
    }
}
