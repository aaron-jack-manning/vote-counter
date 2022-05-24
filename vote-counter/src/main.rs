use std::path;
use std::mem;
use std::env;

/// Represents the result of an election.
enum ElecResult {
    Winner(usize),
    Tie(Vec<usize>),
    StillCounting,
}

/// Collection of candidates, where the index is in the same order as the provided CSV, used
/// throughout the program to identify a candidate.
#[derive(Debug)]
struct Candidates(Vec<String>);

impl Candidates {
    /// Returns the candidates read from the header of a CSV file.
    fn from(path : &path::PathBuf) -> Result<Candidates, csv::Error> {
        let mut reader = csv::Reader::from_path(path)?;
        let headers = reader.headers()?;

        let result : Vec<String> =
            headers
            .into_iter()
            .map(|x| (*x).parse::<String>())
            .map(|x| x.unwrap())
            .collect();

        Ok(Candidates(result))
    }

    /// Gets a candidate's name based on their index.
    fn get(&self, candidate : usize) -> Option<&String> {
        self.0.get(candidate)
    }

    /// Returns the number of candidates.
    fn qty(&self) -> usize {
        self.0.len()
    }
}

/// The index of the inner vector is the preference, with the value stored at that index
/// representing the candidate. Preferences start at 0, and are adjusted at file read.
#[derive(Debug)]
struct Ballot(Vec<usize>);

impl Ballot {
    /// Creates a new ballot from the underlying vector.
    fn new(ballot : Vec<usize>) -> Ballot {
        Ballot(ballot)
    }

    // This processes a ballot as it appears in the provided CSV (as a Vec<Option<usize>> where
    // None means expressing no preference).
    // As such, this function takes a vector indexed by candidate, which yields the preference, and
    // returns a trimmed vector indexed by preference which yields the candidate. 
    fn from_csv_row(csv_row : Vec<Option<usize>>, num_candidates : usize) -> Option<Ballot> {
        // Populate the ballot paper with None before mutating, so that it can be done at arbitrary
        // index.
        let mut ballot = Vec::with_capacity(num_candidates);
        for _i in 0..csv_row.len() {
            ballot.push(None);
        }

        for (candidate, preference) in csv_row.iter().enumerate() {
            // If the preference was expressed.
            if let Some(preference) = preference {
                if preference >= &num_candidates || ballot[*preference].is_some() {
                    // Preference was specified which was greater than the number of candidates or
                    // duplicate preferences were expressed, both of which lead to a discounted
                    // vote.
                    return None;
                }
                else {
                    // No errors, so store the candidate at the index by preference.
                    ballot[*preference] = Some(candidate);
                }
            }
        }

        let ballot =
            ballot
            .iter()
            // Filter out any candidates which were not filled out.
            .filter_map(|x| *x)
            .collect();

        Some(Ballot::new(ballot))
    }
}





/// Trie like structure, which stores ballots with common starting preferences, using the endings
/// value to represent how many votes expressed the preferences from that node to the top.
/// Each 'level' of ballot box represents a preference, from 1 (or 0 internally) down, with each
/// candidate appearing in the children.
#[derive(Debug, Clone)]
struct BallotBox {
    total : u32,
    endings : u32,
    children : Vec<Option<BallotBox>>,
}

impl BallotBox {
    /// Creates a new ballot box.
    fn new(total : u32, endings : u32, children : Vec<Option<BallotBox>>) -> BallotBox {
        BallotBox {
            total,
            endings,
            children
        }
    }

    /// Creates an empty ballot box.
    fn empty(children : usize) -> BallotBox {
        BallotBox::new(0, 0, vec![None; children])
    }

    /// Fills the ballot box from a csv.
    fn fill(path : &path::PathBuf, num_candidates : usize) -> Result<BallotBox, csv::Error> {
        let mut ballot_box = BallotBox::empty(num_candidates);
        let mut reader =
            csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)?;

        for result in reader.records() {
            let mut row = Vec::new();
            for value in result?.iter() {
                // The use of ok() and map() convert the result into an option, so that not
                // expressing preference is none, and then proceed to reduce all by 1 so that the
                // data structures can be 0 indexed even when the preferences start at 1 on the
                // papers themselves.
                row.push(value.parse::<usize>().ok().map(|x| x - 1))
            }

            #[cfg(debug_assertions)]
            let raw_ballot = row.clone();

            // If the ballot can be created from the csv row without error, it should be added to
            // the ballot box.
            if let Some(ballot) = Ballot::from_csv_row(row.clone(), num_candidates) {
                ballot_box.push(ballot, num_candidates);
            }
            else {
                #[cfg(debug_assertions)]
                {
                    println!("INVALID BALLOT: {:?}", raw_ballot);
                }
            }
        }

        Ok(ballot_box)
    }

    /// Wrapper around push many which adds a single ballot to the ballot_box.
    fn push(&mut self, ballot : Ballot, num_candidates : usize) {
        self.push_many(ballot, num_candidates, 1);
    }
    
    /// Pushes quantity many instances of the provided ballot to the ballot box. The ballot is
    /// consumed and freed.
    fn push_many(&mut self, ballot : Ballot, num_candidates : usize, quantity : u32) {
        
        self.total += quantity;

        let mut curr = self;
        
        for (preference, candidate) in ballot.0.iter().enumerate() {

            // If the next candidate is none in the ballot box, that sub box needs to be created.
            if curr.children[*candidate].is_none() {
                curr.children[*candidate] = Some(BallotBox::empty(num_candidates));
            }

            // Traverse the box downwards by entering the new ballot box.
            let children = &mut curr.children;
            curr = children[*candidate].as_mut().unwrap();

            // Update the totals on the current node.
            curr.total += quantity;

            // Reached the end of the ballot paper so need to mark the ending.
            if preference == ballot.0.len() - 1 {
                curr.endings += quantity;
            }
        }
    }

    /// Assuming no winner exists, performs the runoff, redistributing votes of the least popular
    /// candidate.
    fn runoff(&mut self, num_candidates : usize) {

        fn all_min(vec : &Vec<u32>) -> Vec<usize> {
            let mut min = std::u32::MAX;
            let mut minimums = Vec::new();
            for (i, element) in vec.iter().enumerate() {
                if element == &min {
                    minimums.push(i);
                }
                else if element < &min {
                    min = *element;
                    minimums.clear();
                    minimums.push(i);
                }
            }
            minimums
        }

        // Total top preference votes for each candidate.
        let totals =
            self
            .children
            .iter()
            .map(|b| b.as_ref().map(|x| x.total).unwrap_or(std::u32::MAX))
            .collect();

        let to_eliminate = all_min(&totals);

        // Collection of all adjusted votes (with first preference removed) to be distributed.
        // These need to be collected across each candidate which is eliminated to prevent later
        // preferences of votes moved within this round from being counted as top level votes.
        let mut adjusted_votes = Vec::new();

        println!("About to eliminate: {:?}", to_eliminate);
        // Loop over each candidate and eliminate accordingly.
        for candidate in to_eliminate {
            // Separate the ballot box which needs to be distributed from the original so that
            // the mutable and immutable ref can exist at the same time.
            let mut to_distribute = None;
            mem::swap(&mut self.children[candidate], &mut to_distribute);

            // Remove the votes to distribute from the top level total as they will be re-added
            // later.
            self.total -= to_distribute.as_ref().unwrap().total;
            BallotBox::runoff_helper(self, &to_distribute.unwrap(), Vec::with_capacity(num_candidates), num_candidates, &mut adjusted_votes);
        }

        for vote in adjusted_votes {
            let (ballot, quantity) = vote;
            self.push_many(ballot, num_candidates, quantity);
        }
    }

    /// Recursively redistributes votes from to_distribute into original_box.
    fn runoff_helper(original_box : &mut BallotBox, to_distribute : &BallotBox, ballot : Vec<usize>, num_candidates : usize, to_add : &mut Vec<(Ballot, u32)>) {

        for (candidate, child) in to_distribute.children.iter().enumerate() {
            // Only need to process valid ballot box children.
            if let Some(ballot_box) = child {
                // Clone the ballot to pass down so that the vote corresponding with the endings is
                // known.
                let mut new_ballot = ballot.clone();
                new_ballot.push(candidate);

                // Redistribute all children.
                BallotBox::runoff_helper(original_box, ballot_box, new_ballot, num_candidates, to_add);
            }
        }


        // Add the ballots accordingly.
        if to_distribute.endings != 0 {
            //original_box.push_many(Ballot::new(ballot), num_candidates, to_distribute.endings);
            to_add.push((Ballot::new(ballot), to_distribute.endings));
        }
    }

    /// Checks if any candidates have reached the required threshold of first preference votes to
    /// win.
    fn winner(&self, threshold : f64) -> ElecResult {

        fn all_max(vec : &Vec<u32>) -> Vec<(usize, u32)> {
            let mut max = std::u32::MIN;
            let mut maximums = Vec::new();
            for (i, element) in vec.iter().enumerate() {
                if element == &max {
                    maximums.push((i, *element));
                }
                else if element > &max {
                    max = *element;
                    maximums.clear();
                    maximums.push((i, *element));
                }
            }
            maximums 
        }

        // If there is a tie a winner cannot be cal
        let totals =
            self
            .children
            .iter()
            .map(|b| b.as_ref().map(|x| x.total).unwrap_or(std::u32::MIN))
            .collect();

        let winners = all_max(&totals);

        let (_, biggest_count) = winners[0];

        let total_votes : u32 = totals.iter().sum();

        if f64::from(biggest_count) / f64::from(total_votes) >= threshold {
            if winners.len() > 1 {
                ElecResult::Tie(winners.iter().map(|x| x.0).collect())
            }
            else {
                ElecResult::Winner(winners[0].0)
            }
        }
        else {
            ElecResult::StillCounting
        }
    }
}

fn main() {
    if env::args().len() == 3 {
        let path = path::PathBuf::from(env::args().nth(1).unwrap());
        let threshold = env::args().nth(2).unwrap().parse::<f64>().unwrap();

        let candidates = Candidates::from(&path).unwrap();
        let mut ballots = BallotBox::fill(&path, candidates.qty()).unwrap();

        let winners = loop {
            match ballots.winner(threshold) {
                ElecResult::Winner(winner) => break vec![winner],
                ElecResult::Tie(winners) => break winners,
                ElecResult::StillCounting => ballots.runoff(candidates.qty()),
            }
        };

        if winners.len() == 1 {
            let winner = candidates.get(winners[0]).unwrap();
            println!("{} is the elected candidate.", winner);
        }
        else {
            let winners : Vec<&String> =
                winners
                .iter()
                .map(|x| candidates.get(*x).unwrap())
                .collect();

            println!("{:?} are the elected candidates.", winners);
        }
    }
    else {
        println!("Usage: vote-counter <CSV_PATH> <THRESHOLD>");
    }
}
