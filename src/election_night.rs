//! Deterministic fictional election-night count for synthetic regions.

use crate::game_content::RegionProfile;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionTally {
    pub name: String,
    pub electorate: u64,
    pub ballots: u64,
    pub candidate_votes: Vec<u64>,
    pub winner_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElectionNight {
    pub candidates: Vec<String>,
    pub national_votes: Vec<u64>,
    pub total_valid_votes: u64,
    pub regions: Vec<RegionTally>,
}

/// Reveal count sheets in a stable seeded order while preserving both each
/// region's ballot total and the campaign's final national vote shares.
pub fn count_by_region(
    candidates: &[String],
    parties: &[String],
    national_shares: &[f64],
    regions: &[&RegionProfile],
    seed: u64,
) -> Result<ElectionNight, &'static str> {
    if candidates.len() < 2
        || candidates.len() != parties.len()
        || candidates.len() != national_shares.len()
        || regions.is_empty()
        || national_shares
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("election-night inputs have mismatched dimensions or invalid values");
    }
    let share_total: f64 = national_shares.iter().sum();
    if (share_total - 100.0).abs() > 0.001 {
        return Err("national candidate shares must sum to 100");
    }
    let mut normalized_shares: Vec<f64> = national_shares
        .iter()
        .map(|share| share / share_total * 100.0)
        .collect();
    let last_index = normalized_shares.len() - 1;
    let preceding_total: f64 = normalized_shares[..last_index].iter().sum();
    normalized_shares[last_index] = (100.0 - preceding_total).max(0.0);

    let mut rng = CountRng::new(seed);
    let electorate: Vec<u64> = regions
        .iter()
        .enumerate()
        .map(|(index, _)| 26_000 + (index as u64 % 7) * 1_750 + rng.next() % 2_400)
        .collect();
    let turnout: Vec<u64> = electorate
        .iter()
        .map(|voters| ((*voters as f64) * (0.66 + (rng.unit() * 0.14))).round() as u64)
        .collect();
    let total_valid_votes: u64 = turnout.iter().sum();
    let national_votes = apportion(total_valid_votes, &normalized_shares);

    let mut matrix = vec![vec![0.0; candidates.len()]; regions.len()];
    for (region_index, region) in regions.iter().enumerate() {
        let mut preferences: Vec<f64> = normalized_shares
            .iter()
            .enumerate()
            .map(|(candidate_index, share)| {
                let jitter = (rng.unit() - 0.5) * 0.035;
                let local_bonus = if parties[candidate_index] == region.previous_winner_party {
                    1.13
                } else {
                    1.0
                };
                (share / 100.0 * local_bonus * (1.0 + jitter)).max(0.000_001)
            })
            .collect();
        let preference_total: f64 = preferences.iter().sum();
        for (candidate_index, preference) in preferences.iter_mut().enumerate() {
            matrix[region_index][candidate_index] =
                turnout[region_index] as f64 * *preference / preference_total;
        }
    }

    // Iterative proportional fitting moves local lean around without changing
    // the national forecast or total ballots in each synthetic region.
    for _ in 0..64 {
        for (row, target) in matrix.iter_mut().zip(&turnout) {
            let total: f64 = row.iter().sum();
            if total > 0.0 {
                let scale = *target as f64 / total;
                for cell in row {
                    *cell *= scale;
                }
            }
        }
        for candidate_index in 0..candidates.len() {
            let total: f64 = matrix.iter().map(|row| row[candidate_index]).sum();
            if total > 0.0 {
                let scale = national_votes[candidate_index] as f64 / total;
                for row in &mut matrix {
                    row[candidate_index] *= scale;
                }
            }
        }
    }

    let mut integer_matrix: Vec<Vec<u64>> = matrix
        .iter()
        .map(|row| row.iter().map(|value| value.floor() as u64).collect())
        .collect();
    let mut row_left: Vec<u64> = turnout
        .iter()
        .enumerate()
        .map(|(row_index, target)| target.saturating_sub(integer_matrix[row_index].iter().sum()))
        .collect();
    let mut column_left: Vec<u64> = (0..candidates.len())
        .map(|candidate_index| {
            national_votes[candidate_index]
                .saturating_sub(integer_matrix.iter().map(|row| row[candidate_index]).sum())
        })
        .collect();
    let mut fractions: Vec<(usize, usize, f64)> = matrix
        .iter()
        .enumerate()
        .flat_map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .map(move |(column, value)| (row, column, value.fract()))
        })
        .collect();
    fractions.sort_by(|left, right| {
        right
            .2
            .total_cmp(&left.2)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
    });
    for (row, column, _) in fractions {
        if row_left[row] > 0 && column_left[column] > 0 {
            integer_matrix[row][column] += 1;
            row_left[row] -= 1;
            column_left[column] -= 1;
        }
    }
    // IPF approaches exact margins iteratively, so floating-point residue can
    // leave a rare rounding tie with no remaining fractional cell. Preserve
    // exact published margins as the primary invariant and use the closest
    // local cells for any final correction.
    for row in 0..integer_matrix.len() {
        let mut columns: Vec<_> = (0..candidates.len())
            .map(|column| (column, matrix[row][column].fract()))
            .collect();
        columns.sort_by(|left, right| {
            right
                .1
                .total_cmp(&left.1)
                .then_with(|| left.0.cmp(&right.0))
        });
        for (column, _) in columns {
            if row_left[row] == 0 {
                break;
            }
            let correction = row_left[row].min(column_left[column]);
            integer_matrix[row][column] += correction;
            row_left[row] -= correction;
            column_left[column] -= correction;
        }
    }
    if row_left.iter().any(|left| *left != 0) || column_left.iter().any(|left| *left != 0) {
        return Err("could not reconcile the regional count sheets with national totals");
    }

    let regions = regions
        .iter()
        .enumerate()
        .map(|(index, region)| {
            let candidate_votes = integer_matrix[index].clone();
            let winner_index = candidate_votes
                .iter()
                .enumerate()
                .max_by(|(left_i, left), (right_i, right)| {
                    left.cmp(right).then_with(|| right_i.cmp(left_i))
                })
                .map(|(winner, _)| winner)
                .unwrap_or(0);
            RegionTally {
                name: region.name.to_string(),
                electorate: electorate[index],
                ballots: turnout[index],
                candidate_votes,
                winner_index,
            }
        })
        .collect();
    Ok(ElectionNight {
        candidates: candidates.to_vec(),
        national_votes,
        total_valid_votes,
        regions,
    })
}

fn apportion(total: u64, shares: &[f64]) -> Vec<u64> {
    let mut counts: Vec<u64> = shares
        .iter()
        .map(|share| (total as f64 * share / 100.0).floor() as u64)
        .collect();
    let mut remainder_order: Vec<(usize, f64)> = shares
        .iter()
        .enumerate()
        .map(|(index, share)| {
            let exact = total as f64 * share / 100.0;
            (index, exact - exact.floor())
        })
        .collect();
    remainder_order.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    let assigned: u64 = counts.iter().sum();
    for (index, _) in remainder_order
        .into_iter()
        .take((total - assigned) as usize)
    {
        counts[index] += 1;
    }
    counts
}

#[derive(Clone, Copy)]
struct CountRng(u64);

impl CountRng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x454c_4543_544e_4947)
    }

    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1_u64 << 53) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_content::REGIONS;

    fn fixture() -> ElectionNight {
        count_by_region(
            &[
                "甲候選人".to_string(),
                "乙候選人".to_string(),
                "丙候選人".to_string(),
            ],
            &[
                "群島共治黨".to_string(),
                "海岳協作黨".to_string(),
                "地方共好聯盟".to_string(),
            ],
            &[43.25, 39.5, 17.25],
            &REGIONS.iter().collect::<Vec<_>>(),
            1729,
        )
        .unwrap()
    }

    #[test]
    fn regional_count_sums_to_campaign_totals_and_each_polling_place() {
        let result = fixture();
        assert_eq!(result.regions.len(), 22);
        assert_eq!(
            result.regions.iter().map(|r| r.ballots).sum::<u64>(),
            result.total_valid_votes
        );
        for (index, total) in result.national_votes.iter().enumerate() {
            assert_eq!(
                result
                    .regions
                    .iter()
                    .map(|r| r.candidate_votes[index])
                    .sum::<u64>(),
                *total
            );
        }
        for region in &result.regions {
            assert_eq!(region.candidate_votes.iter().sum::<u64>(), region.ballots);
            assert!(region.winner_index < result.candidates.len());
        }
    }

    #[test]
    fn fixed_seed_reproduces_the_same_count() {
        assert_eq!(fixture(), fixture());
    }

    #[test]
    fn near_100_percent_inputs_are_normalized_before_reconciliation() {
        let result = count_by_region(
            &["甲候選人".to_string(), "乙候選人".to_string()],
            &["群島共治黨".to_string(), "海岳協作黨".to_string()],
            &[60.0008, 40.0],
            &REGIONS.iter().collect::<Vec<_>>(),
            91,
        )
        .unwrap();
        for (index, national) in result.national_votes.iter().enumerate() {
            assert_eq!(
                result
                    .regions
                    .iter()
                    .map(|region| region.candidate_votes[index])
                    .sum::<u64>(),
                *national
            );
        }
    }

    #[test]
    fn regional_reconciliation_is_stable_across_seed_range() {
        let candidates = ["甲候選人".to_string(), "乙候選人".to_string()];
        let parties = ["群島共治黨".to_string(), "海岳協作黨".to_string()];
        let regions: Vec<_> = REGIONS.iter().collect();
        for seed in 0..256 {
            let result =
                count_by_region(&candidates, &parties, &[52.5, 47.5], &regions, seed).unwrap();
            assert_eq!(
                result
                    .regions
                    .iter()
                    .map(|region| region.ballots)
                    .sum::<u64>(),
                result.total_valid_votes
            );
            for index in 0..candidates.len() {
                assert_eq!(
                    result
                        .regions
                        .iter()
                        .map(|region| region.candidate_votes[index])
                        .sum::<u64>(),
                    result.national_votes[index]
                );
            }
        }
    }
}
