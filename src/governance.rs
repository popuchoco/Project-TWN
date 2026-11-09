//! Government approval and issue evaluation are distinct from candidate vote
//! intention and campaign-team trust. Values are game-state indicators, not polls.

use crate::history::Issue;

pub const ISSUE_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GovernmentOpinion {
    pub administration_approval: f64,
    pub issue_approval: [f64; ISSUE_COUNT],
    pub issue_salience: [f64; ISSUE_COUNT],
    pub observations: u16,
}

impl Default for GovernmentOpinion {
    fn default() -> Self {
        Self {
            administration_approval: 50.0,
            issue_approval: [50.0; ISSUE_COUNT],
            issue_salience: [10.0; ISSUE_COUNT],
            observations: 0,
        }
    }
}

impl GovernmentOpinion {
    pub fn observe(
        &mut self,
        issue: Issue,
        approval_delta: f64,
        issue_delta: f64,
        salience_delta: f64,
    ) {
        if !approval_delta.is_finite() || !issue_delta.is_finite() || !salience_delta.is_finite() {
            return;
        }
        let index = issue_index(issue);
        self.administration_approval =
            (self.administration_approval + approval_delta).clamp(0.0, 100.0);
        self.issue_approval[index] = (self.issue_approval[index] + issue_delta).clamp(0.0, 100.0);
        self.issue_salience[index] =
            (self.issue_salience[index] + salience_delta).clamp(0.0, 100.0);
        self.observations = self.observations.saturating_add(1);
    }

    pub fn issue_rating(&self, issue: Issue) -> f64 {
        self.issue_approval[issue_index(issue)]
    }

    pub fn issue_priority(&self, issue: Issue) -> f64 {
        self.issue_salience[issue_index(issue)]
    }

    pub fn most_salient_issue(&self) -> Issue {
        let mut index = 0;
        for candidate in 1..ISSUE_COUNT {
            if self.issue_salience[candidate] > self.issue_salience[index] {
                index = candidate;
            }
        }
        issue_from_index(index)
    }
}

pub const fn issue_index(issue: Issue) -> usize {
    match issue {
        Issue::Economy => 0,
        Issue::Trust => 1,
        Issue::CrossStrait => 2,
        Issue::Media => 3,
        Issue::Civic => 4,
        Issue::Education => 5,
        Issue::Rights => 6,
        Issue::PublicHealth => 7,
        Issue::Governance => 8,
        Issue::Local => 9,
    }
}

pub const fn issue_from_index(index: usize) -> Issue {
    match index {
        0 => Issue::Economy,
        1 => Issue::Trust,
        2 => Issue::CrossStrait,
        3 => Issue::Media,
        4 => Issue::Civic,
        5 => Issue::Education,
        6 => Issue::Rights,
        7 => Issue::PublicHealth,
        8 => Issue::Governance,
        _ => Issue::Local,
    }
}

pub const fn issue_label(issue: Issue) -> &'static str {
    match issue {
        Issue::Economy => "經濟與生活成本",
        Issue::Trust => "政治信任與誠信",
        Issue::CrossStrait => "兩岸與區域安全",
        Issue::Media => "媒體與資訊環境",
        Issue::Civic => "公共參與",
        Issue::Education => "教育與課綱",
        Issue::Rights => "權利與平等",
        Issue::PublicHealth => "公共健康",
        Issue::Governance => "政府治理",
        Issue::Local => "地方服務",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn government_and_issue_opinions_are_bounded_and_independent() {
        let mut opinion = GovernmentOpinion::default();
        opinion.observe(Issue::Economy, 1.5, -3.0, 4.0);
        assert_eq!(opinion.administration_approval, 51.5);
        assert_eq!(opinion.issue_rating(Issue::Economy), 47.0);
        assert_eq!(opinion.issue_rating(Issue::Rights), 50.0);
        assert_eq!(opinion.issue_priority(Issue::Economy), 14.0);
        assert_eq!(opinion.observations, 1);

        opinion.observe(Issue::Economy, 100.0, 100.0, 100.0);
        assert_eq!(opinion.administration_approval, 100.0);
        assert_eq!(opinion.issue_rating(Issue::Economy), 100.0);
        assert_eq!(opinion.issue_priority(Issue::Economy), 100.0);
    }

    #[test]
    fn issue_indices_round_trip_and_salience_tie_is_stable() {
        for index in 0..ISSUE_COUNT {
            assert_eq!(issue_index(issue_from_index(index)), index);
        }
        let mut opinion = GovernmentOpinion::default();
        opinion.observe(Issue::Rights, 0.0, 0.0, 1.0);
        assert_eq!(opinion.most_salient_issue(), Issue::Rights);
    }
}
