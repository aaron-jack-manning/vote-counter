use std::mem;
use std::path;

use crate::candidates::Candidates;
use crate::reporting;
use crate::ballot::Ballot;

/// Represents the current status of the count, and how to proceed counting.
#[derive(Clone, Debug)]
pub enum CountStatus {
    Winner(usize),
    Tie,
    Promotion(Vec<usize>),
    Runoff(Vec<usize>),
}

#[derive(Debug, Clone)]
/// Node of trie like structure representing the votes. This stores ballots with common starting
/// preference, using the endings value to count how many votes expressed the same preference from
/// the top to that node. Each 'level' of the structure represents a preference, with each
/// candidate appearing in the `children` field's vector in order.
struct BallotBoxNode {
    total_beneath : u32,
    endings : u32,
    children : Vec<Option<BallotBoxNode>>,
}

impl BallotBoxNode {
    /// Creates a new, empty ballot box node.
    fn new(children : usize) -> Self {
        BallotBoxNode {
            total_beneath : 0,
            endings : 0,
            children : vec![None; children],
        }
    }
}

/// Stores list of candidates, total number of votes, the candidates which have been eliminated and
/// the votes themselves using a `BallotBoxNode`s.
#[derive(Debug, Clone)]
pub struct BallotBox {
    eliminated : Vec<bool>,
    total_votes : u32,
    nodes : Vec<Option<BallotBoxNode>>,
    pub candidates : Candidates,
}

impl BallotBox {
    /// Creates a new, empty ballot box.
    fn new(candidates : Candidates) -> Self {
        BallotBox {
            eliminated : vec![true; candidates.len()],
            total_votes : 0,
            nodes : vec![None; candidates.len()],
            candidates,
        }
    }

    /// Reads and fills the ballot box from a file.
    pub fn from_file(path : &path::PathBuf, report : bool) -> Result<BallotBox, csv::Error> {

        let mut reader =
            csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        // Read the headers and create the candidates.
        let headers = reader.headers()?;

        let candidates : Vec<String> =
            headers
            .into_iter()
            .map(|x| (*x).parse::<String>())
            .map(|x| x.unwrap())
            .collect();

        let candidates = Candidates::new(candidates);

        let mut ballot_box = BallotBox::new(candidates);

        let mut counter = 1;
        for result in reader.records() {
            let mut raw_ballot = Vec::new();
            counter += 1;

            for value in result?.iter() {
                raw_ballot.push(value.parse::<usize>().ok())
            }

            match Ballot::from_raw_ballot(raw_ballot) {
                Ok(ballot) => ballot_box.push(ballot, 1),
                Err(raw_ballot) => reporting::invalid_ballot(counter, &raw_ballot, report),
            }
        }

        Ok(ballot_box)
    }

    /// Returns a collection of all eliminated candidates.
    fn eliminated(&self) -> Vec<usize> {
        let mut eliminated = Vec::new();

        for i in 0..self.candidates.len() {
            if self.eliminated[i] {
                eliminated.push(i)
            }
        }

        eliminated
    }

    /// Returns the number of remaining candidates which have yet to be eliminated.
    fn remaining(&self) -> usize {
        self
        .eliminated
        .iter()
        .filter(|b| !*b)
        .count()
    }

    /// Adds the provided ballot to the `BallotBox` `quantity` times.
    fn push(&mut self, ballot : Ballot, quantity : u32) {

        // All candidates are marked as eliminated at the start, so this may need to change as each
        // new ballot is added in.
        self.eliminated[ballot.first_pref()] = false;

        // Update the total number of votes at the top level.
        self.total_votes += quantity;

        let mut current_node : Option<&mut BallotBoxNode> = None;
        
        for (_, &candidate) in ballot.iter().enumerate() {

            // Traverse down the trie appropriately depending on if it is currently at the top
            // level or not.
            current_node = match current_node {
                None => {
                    if self.nodes[candidate].is_none() {
                        self.nodes[candidate] = Some(BallotBoxNode::new(self.candidates.len()));
                    }

                    let children = &mut self.nodes;
                    Some(children[candidate].as_mut().unwrap())
                },
                Some(current_node) => {
                    if current_node.children[candidate].is_none() {
                        current_node.children[candidate] = Some(BallotBoxNode::new(self.candidates.len()));
                    }

                    let children = &mut current_node.children;
                    Some(children[candidate].as_mut().unwrap())
                }
            };

            // Update the total number of votes under the current node.
            current_node.as_mut().unwrap().total_beneath += quantity;
        }

        // Update the endings count on the last node.
        current_node.unwrap().endings += quantity;
    }


    // Gives the current status of the count, and indicates who needs to be eliminated in a runoff
    // if necessary.
    pub fn status(&self, threshold : f64, report : bool) -> CountStatus {
        let totals : Vec<u32> =
            self
            .nodes
            .iter()
            .map(|n| match n {
                None => 0,
                Some(node) => node.total_beneath,
            })
            .collect();

        let max = *totals.iter().max().unwrap();
        let min = *totals.iter().filter(|x| x != &&0).min().unwrap();

        let winners =
            totals
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut winners, (candidate, total)| {
                if total == &max {
                    winners.push(candidate);
                };

                winners
            });

        let losers = 
            totals
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut losers, (candidate, total)| {
                if total == &min {
                    losers.push(candidate);
                };

                losers 
            });

        reporting::current_count(totals.iter().enumerate().map(|(a, b)| (a, *b)).collect(), &self.candidates, report);

        // All votes have been reduced to 0.
        let status = if max == 0 {
            CountStatus::Tie
        }
        // A unique winner has been determined.
        else if winners.len() == 1 && f64::try_from(max).unwrap() >= (threshold * f64::try_from(self.total_votes).unwrap()) {
            CountStatus::Winner(winners[0])
        }
        // All remaining candidates are on equal votes.
        else if winners.len() == self.remaining() {
            CountStatus::Promotion(winners)
        }
        // Distribute the votes of all losers.
        else {
            CountStatus::Runoff(losers)
        };

        reporting::status(&status, &self.candidates, report);

        status
    }

    /// Promotes lower preference votes of the provided candidates.
    pub fn promote(&mut self, to_promote : Vec<usize>) {
        self.runoff_or_promote(to_promote, false);
    }

    /// Eliminates the provided candidates and distributes their votes.
    pub fn runoff(&mut self, to_eliminate : Vec<usize>) {
        self.runoff_or_promote(to_eliminate, true);
    }

    fn runoff_or_promote(&mut self, to_promote_or_eliminate : Vec<usize>, runoff : bool) {
        // Vector of ballots and the quantity to redistribute.
        let mut adjusted_votes : Vec<(Ballot, u32)> = Vec::new();

        for candidate in to_promote_or_eliminate {
            // Swap the votes to distribute out.
            let mut to_distribute = None;
            mem::swap(&mut self.nodes[candidate], &mut to_distribute);
            let to_distribute = to_distribute.unwrap();

            // Update the top level total.
            self.total_votes -= to_distribute.total_beneath;
            
            BallotBox::distribute(&to_distribute, Vec::new(), &mut adjusted_votes);

            // Update the array of eliminated candidates.
            if runoff {
                self.eliminated[candidate] = true;
            }
        }

        // Determine all previously eliminated candidates (including in this round).
        let eliminated_candidates : Vec<usize> = self.eliminated();

        for (vote, qty) in adjusted_votes {
            // Remove any preferences expressed for the candidates which have already been
            // eliminated, and add the remaining ballot if it is non-empty.
            if let Some(vote) = Ballot::remove_candidates(vote, &eliminated_candidates) {
                self.push(vote, qty);
            }
        }
    }

    /// Helper function for `runoff_or_promote` which handles the calculating of votes that need to
    /// be distributed.
    fn distribute(to_distribute : &BallotBoxNode, current_ballot : Vec<usize>, adjusted_votes : &mut Vec<(Ballot, u32)>) {
        for (candidate, child) in to_distribute.children.iter().enumerate() {
            if let Some(node) = child {
                // Clone the current ballot so that new values can be added as passed down.
                let mut next_ballot = current_ballot.clone();
                // Add the current candidate to the ballot.
                next_ballot.push(candidate);

                BallotBox::distribute(node, next_ballot, adjusted_votes);
            }
        }

        // Add the current ballot to the collection with the corresponding count.
        // This will intentionally ignore ballots at the top level, which are being distributed
        // anyway.
        if to_distribute.endings > 0 {
            adjusted_votes.push((Ballot::new(current_ballot), to_distribute.endings));
        }
    }
}


