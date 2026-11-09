//! Fictional party-ticket modelling and proportional seat conversion.
//! The configured seat count and threshold model the current national party-list
//! framework; they are not a forecast or a reproduction of historical results.

use std::collections::BTreeMap;

pub const PARTY_LIST_SEATS: usize = 34;
pub const PARTY_LIST_THRESHOLD_PCT: f64 = 5.0;

#[derive(Clone, Debug, PartialEq)]
pub struct PartyTicket {
    pub party: String,
    pub support_pct: f64,
    pub vote_share_pct: f64,
    pub votes: u64,
    pub seats: usize,
}

/// Blend party identity and candidate support, then apply small campaign signals.
/// The output remains deliberately distinct from candidate-level support.
pub fn party_ticket_support(
    candidates: &[(String, String, f64, f64)],
    player_party: &str,
    trust: f64,
    mobilization: f64,
) -> Result<Vec<PartyTicket>, &'static str> {
    if candidates.len() < 2
        || candidates.iter().any(|(_, _, starting, current)| {
            !starting.is_finite() || !current.is_finite() || *starting < 0.0 || *current < 0.0
        })
    {
        return Err("candidate support input must contain at least two valid candidates");
    }
    let mut grouped: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for (party, _, starting, current) in candidates {
        let entry = grouped.entry(party.clone()).or_default();
        entry.0 += starting;
        entry.1 += current;
    }
    let mut values: Vec<_> = grouped
        .into_iter()
        .map(|(party, (starting, current))| {
            let identity_and_candidate = starting * 0.65 + current * 0.35;
            let campaign_signal = if party == player_party {
                (trust - 50.0) * 0.025 + (mobilization - 50.0) * 0.015
            } else {
                0.0
            };
            (party, (identity_and_candidate + campaign_signal).max(0.0))
        })
        .collect();
    let sum: f64 = values.iter().map(|(_, value)| value).sum();
    if !sum.is_finite() || sum <= 0.0 {
        return Err("party support must have a positive finite total");
    }
    Ok(values
        .drain(..)
        .map(|(party, value)| PartyTicket {
            party,
            support_pct: value / sum * 100.0,
            vote_share_pct: 0.0,
            votes: 0,
            seats: 0,
        })
        .collect())
}

/// Convert party-ticket support to list votes and allocate seats using the
/// largest-remainder method after applying the statutory party threshold.
pub fn allocate_party_list_seats(
    mut tickets: Vec<PartyTicket>,
    total_valid_votes: u64,
    seat_count: usize,
    threshold_pct: f64,
) -> Result<Vec<PartyTicket>, &'static str> {
    if tickets.is_empty()
        || total_valid_votes == 0
        || seat_count == 0
        || !threshold_pct.is_finite()
        || !(0.0..=100.0).contains(&threshold_pct)
        || tickets
            .iter()
            .any(|ticket| !ticket.support_pct.is_finite() || ticket.support_pct < 0.0)
    {
        return Err("seat allocation input is outside supported bounds");
    }
    let support_total: f64 = tickets.iter().map(|ticket| ticket.support_pct).sum();
    if !support_total.is_finite() || support_total <= 0.0 {
        return Err("party support must have a positive finite total");
    }
    let normalized_support: Vec<f64> = tickets
        .iter()
        .map(|ticket| ticket.support_pct / support_total * 100.0)
        .collect();
    for (ticket, share) in tickets.iter_mut().zip(&normalized_support) {
        ticket.support_pct = *share;
    }
    let vote_counts = apportion(total_valid_votes, &normalized_support);
    for (ticket, votes) in tickets.iter_mut().zip(vote_counts) {
        ticket.votes = votes;
    }

    let eligible: Vec<usize> = tickets
        .iter()
        .enumerate()
        .filter(|(_, ticket)| ticket.support_pct + f64::EPSILON >= threshold_pct)
        .map(|(index, _)| index)
        .collect();
    if eligible.is_empty() {
        return Err("no party meets the configured party-list threshold");
    }
    let eligible_total: f64 = eligible
        .iter()
        .map(|index| tickets[*index].support_pct)
        .sum();
    let mut remainders = Vec::with_capacity(eligible.len());
    let mut allocated = 0;
    for index in &eligible {
        let exact = tickets[*index].support_pct / eligible_total * seat_count as f64;
        let seats = exact.floor() as usize;
        tickets[*index].seats = seats;
        allocated += seats;
        remainders.push((*index, exact - seats as f64));
        tickets[*index].vote_share_pct = tickets[*index].support_pct;
    }
    remainders.sort_by(|(left_index, left), (right_index, right)| {
        right
            .total_cmp(left)
            .then_with(|| tickets[*left_index].party.cmp(&tickets[*right_index].party))
    });
    for (index, _) in remainders.into_iter().take(seat_count - allocated) {
        tickets[index].seats += 1;
    }
    for (index, ticket) in tickets.iter_mut().enumerate() {
        if !eligible.contains(&index) {
            ticket.vote_share_pct = ticket.support_pct;
        }
    }
    Ok(tickets)
}

fn apportion(total: u64, shares: &[f64]) -> Vec<u64> {
    let mut counts: Vec<u64> = shares
        .iter()
        .map(|share| (total as f64 * share / 100.0).floor() as u64)
        .collect();
    let mut remainders: Vec<_> = shares
        .iter()
        .enumerate()
        .map(|(index, share)| {
            let exact = total as f64 * share / 100.0;
            (index, exact - exact.floor())
        })
        .collect();
    remainders.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let assigned: u64 = counts.iter().sum();
    for (index, _) in remainders.into_iter().take((total - assigned) as usize) {
        counts[index] += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_support_is_separate_from_personal_poll_and_rewards_campaign_quality() {
        let candidates = vec![
            ("群島共治黨".to_string(), "甲".to_string(), 40.0, 52.0),
            ("海岳協作黨".to_string(), "乙".to_string(), 35.0, 30.0),
            ("地方共好聯盟".to_string(), "丙".to_string(), 25.0, 18.0),
        ];
        let tickets = party_ticket_support(&candidates, "群島共治黨", 80.0, 70.0).unwrap();
        assert!((tickets.iter().map(|t| t.support_pct).sum::<f64>() - 100.0).abs() < 1e-9);
        let governing_ticket = tickets
            .iter()
            .find(|ticket| ticket.party == "群島共治黨")
            .unwrap();
        assert!(governing_ticket.support_pct > 40.0);
        assert_ne!(governing_ticket.support_pct, 52.0);
    }

    #[test]
    fn list_seats_obey_threshold_and_sum_to_configured_total() {
        let tickets = [49.0, 31.0, 15.1, 4.9]
            .into_iter()
            .enumerate()
            .map(|(index, support_pct)| PartyTicket {
                party: format!("黨{index}"),
                support_pct,
                vote_share_pct: 0.0,
                votes: 0,
                seats: 0,
            })
            .collect();
        let result = allocate_party_list_seats(tickets, 1_000_000, 34, 5.0).unwrap();
        assert_eq!(result.iter().map(|ticket| ticket.seats).sum::<usize>(), 34);
        assert_eq!(result[3].seats, 0);
        assert_eq!(
            result[0].votes + result[1].votes + result[2].votes + result[3].votes,
            1_000_000
        );
    }

    #[test]
    fn tied_remainders_are_stable_by_party_name() {
        let tickets = ["丙黨", "甲黨", "乙黨"]
            .into_iter()
            .map(|party| PartyTicket {
                party: party.to_string(),
                support_pct: 100.0 / 3.0,
                vote_share_pct: 0.0,
                votes: 0,
                seats: 0,
            })
            .collect();
        let result = allocate_party_list_seats(tickets, 99, 2, 0.0).unwrap();
        let mut winners: Vec<_> = result
            .iter()
            .filter(|ticket| ticket.seats == 1)
            .map(|ticket| ticket.party.as_str())
            .collect();
        winners.sort_unstable();
        assert_eq!(winners, vec!["丙黨", "乙黨"]);
    }
}
