use std::collections::HashSet;

/// Represents a ballot paper.
#[derive(Debug, Clone)]
pub struct Ballot(Vec<usize>);

impl Ballot {
    /// Creates a new ballot from the a `Vec<usize> where the values within the `Vec` are
    /// candidates and the order within the `Vec` expresses preference.
    pub fn new(ballot : Vec<usize>) -> Ballot {
        Ballot(ballot)
    }

    /// Creates an iterator over the undelying `Vec<usize>` within the `Ballot`.
    pub fn iter(&self) -> std::slice::Iter<'_, usize> {
        self.0.iter()
    }

    /// Removes the specified candidates from the ballot.
    pub fn remove_candidates(ballot : Ballot, to_remove : &[usize]) -> Option<Ballot> {
        let new_ballot: Vec<_> = 
            ballot.0
            .into_iter()
            .filter(|c| !to_remove.contains(c))
            .collect();

        match new_ballot.len() {
            0 => None,
            _ => Some(Ballot::new(new_ballot))
        }
    }

    /// Returns the highest preference candidate.
    pub fn first_pref(&self) -> usize {
        self.0[0]
    }

    /// Creates a ballot from the representation read from the file.
    pub fn from_raw_ballot(raw_ballot : Vec<Option<usize>>) -> Result<Ballot, Vec<Option<usize>>> {
        let mut pref_pairs = Vec::with_capacity(raw_ballot.len());

        let mut preference_set = HashSet::with_capacity(raw_ballot.len());

        for (candidate, preference) in raw_ballot.iter().enumerate() {
            if let Some(preference) = preference {
                if !preference_set.insert(preference) {
                    // Value already existed in set, which means preference was expressed twice.
                    return Err(raw_ballot);
                }
                pref_pairs.push((preference, candidate));
            }
        }

        match pref_pairs.len() {
            // No preference was expressed at all.
            0 => Err(raw_ballot),
            _ => {
                // Sort the ballot by order of preference.
                pref_pairs.sort_by(|(p1, _), (p2, _)| p1.cmp(p2));

                // Resolve the preference-candidate pairs to just the candidate.
                let ballot =
                    pref_pairs
                    .into_iter()
                    .map(|(_, c)| c)
                    .collect();

                Ok(Ballot(ballot))
            }
        }
    }
}
