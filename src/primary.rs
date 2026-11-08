use crate::sim::CampaignStyle;

pub const PRIMARY_ROUNDS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryChoice {
    PolicyDebate,
    OrganizerVisit,
    CommunicationForum,
    ComparisonPoll,
    UnityNegotiation,
}

impl PrimaryChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::PolicyDebate => "政策論壇（政策派）",
            Self::OrganizerVisit => "地方組織行程（組織派）",
            Self::CommunicationForum => "公開溝通活動（溝通派）",
            Self::ComparisonPoll => "對比／互比式民調",
            Self::UnityNegotiation => "初選整合協商",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrimaryContest {
    pub polls: [f64; 3],
    pub round: u8,
    seed: u64,
}

impl PrimaryContest {
    pub fn new(seed: u64) -> Self {
        let mut contest = Self {
            polls: [36.0, 34.0, 30.0],
            round: 0,
            seed: if seed == 0 {
                0x0050_5249_4D41_5259
            } else {
                seed
            },
        };
        let variation = contest.random_between(-1.5, 1.5);
        shift(&mut contest.polls, 2, variation);
        contest
    }

    pub fn apply(&mut self, choice: PrimaryChoice) -> Result<(), String> {
        if self.round >= PRIMARY_ROUNDS {
            return Err("黨內初選已結束。".to_string());
        }
        match choice {
            PrimaryChoice::PolicyDebate => {
                let delta = self.random_between(2.0, 3.2);
                shift(&mut self.polls, 0, delta);
            }
            PrimaryChoice::OrganizerVisit => {
                let delta = self.random_between(2.0, 3.2);
                shift(&mut self.polls, 1, delta);
            }
            PrimaryChoice::CommunicationForum => {
                let delta = self.random_between(2.0, 3.2);
                shift(&mut self.polls, 2, delta);
            }
            PrimaryChoice::ComparisonPoll => {
                // The two hypothetical head-to-head comparisons make the
                // leading contender slightly more viable with undecideds.
                let leader = winner(&self.polls);
                let delta = self.random_between(0.6, 1.1);
                shift(&mut self.polls, leader, delta);
            }
            PrimaryChoice::UnityNegotiation => {
                let leader = winner(&self.polls);
                let trailing = weakest_except(&self.polls, leader);
                let amount = self.polls[trailing] * self.random_between(0.06, 0.12);
                transfer(&mut self.polls, trailing, leader, amount);
            }
        }
        self.round += 1;
        normalize(&mut self.polls);
        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.round >= PRIMARY_ROUNDS
    }

    pub fn nominee_index(&self) -> usize {
        winner(&self.polls)
    }

    pub fn nominee_style(&self) -> CampaignStyle {
        match self.nominee_index() {
            0 => CampaignStyle::Policy,
            1 => CampaignStyle::Organizer,
            _ => CampaignStyle::Communicator,
        }
    }

    pub fn name(index: usize) -> &'static str {
        ["政策派", "組織派", "溝通派"][index]
    }

    pub fn head_to_head(&self) -> [(usize, usize, f64); 3] {
        [(0, 1, 0.0), (0, 2, 0.0), (1, 2, 0.0)].map(|(left, right, _)| {
            let remaining = (100.0 - self.polls[left] - self.polls[right]).max(0.0);
            let left_share = (self.polls[left] + remaining * 0.5) / 100.0 * 100.0;
            (left, right, left_share)
        })
    }

    fn random_between(&mut self, low: f64, high: f64) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        low + (high - low) * ((self.seed >> 11) as f64 / ((1_u64 << 53) as f64))
    }
}

fn winner(polls: &[f64; 3]) -> usize {
    polls
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn weakest_except(polls: &[f64; 3], excluded: usize) -> usize {
    polls
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != excluded)
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn shift(polls: &mut [f64; 3], candidate: usize, delta: f64) {
    let old = polls[candidate];
    let new = (old + delta).clamp(0.0, 100.0);
    let actual = new - old;
    let other_total: f64 = polls
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != candidate)
        .map(|(_, share)| *share)
        .sum();
    polls[candidate] = new;
    if other_total > f64::EPSILON {
        for (index, share) in polls.iter_mut().enumerate() {
            if index != candidate {
                *share -= actual * *share / other_total;
            }
        }
    }
    normalize(polls);
}

fn transfer(polls: &mut [f64; 3], from: usize, to: usize, requested: f64) {
    if from == to || requested <= 0.0 {
        return;
    }
    let amount = requested.min(polls[from]);
    polls[from] -= amount;
    polls[to] += amount;
    normalize(polls);
}

fn normalize(polls: &mut [f64; 3]) {
    let total: f64 = polls.iter().sum();
    if total > f64::EPSILON {
        for share in polls {
            *share = *share / total * 100.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_runs_four_rounds_and_selects_a_style() {
        let mut contest = PrimaryContest::new(7);
        for choice in [
            PrimaryChoice::PolicyDebate,
            PrimaryChoice::UnityNegotiation,
            PrimaryChoice::ComparisonPoll,
            PrimaryChoice::OrganizerVisit,
        ] {
            contest.apply(choice).unwrap();
        }
        assert!(contest.is_finished());
        assert!((contest.polls.iter().sum::<f64>() - 100.0).abs() < 0.0001);
        assert!(matches!(
            contest.nominee_style(),
            CampaignStyle::Policy | CampaignStyle::Organizer | CampaignStyle::Communicator
        ));
        assert!(contest.apply(PrimaryChoice::PolicyDebate).is_err());
    }

    #[test]
    fn head_to_head_results_are_complements() {
        let contest = PrimaryContest::new(17);
        for (left, right, left_share) in contest.head_to_head() {
            let remainder = 100.0 - contest.polls[left] - contest.polls[right];
            let right_share = contest.polls[right] + remainder * 0.5;
            assert!((left_share + right_share - 100.0).abs() < 0.0001);
        }
    }
}
