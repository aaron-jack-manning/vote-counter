use colored::*;

use crate::ballot_box::{
    CountStatus,
    CountStatus::*
};
use crate::candidates::Candidates;

/// Displays the invalid ballot provided.
pub fn invalid_ballot(number : u32, ballot : &[Option<usize>], report : bool) {
    if report {
        let segments : Vec<_> =
            ballot
            .iter()
            .map(|op| {
                match op {
                    None => String::from("_"),
                    Some(pref) => pref.to_string(),
                }
            })
            .collect();

        let formatted = segments.join(",");
        println!("{} {} (line: {})", "Invalid Ballot:".bright_green().bold(), formatted, number);
    }
}

/// Displays the current count of top preference votes.   
pub fn current_count(count : Vec<(usize, u32)>, candidates : &Candidates, report : bool) {
    if report {
        println!("{}", "Current Count:".bright_yellow().bold());

        for (candidate, votes) in count {
            println!("    {} : {}", candidates.get(candidate).unwrap(), votes);
        }
    }
}

/// Displays a `CountStatus` and associated data if it is a `Runoff` or `Promotion`.
pub fn status(status : &CountStatus, candidates : &Candidates, report : bool) {
    if report {
        match status {
            Runoff(to_distribute) => {
                let candidates = to_distribute.iter().map(|c| candidates.get(*c).unwrap().clone()).collect::<Vec<String>>().join(", ");
                println!("{} {}", "Eliminating:".bright_magenta(), candidates);
            },
            Promotion(to_promote) => {
                let candidates = to_promote.iter().map(|c| candidates.get(*c).unwrap().clone()).collect::<Vec<String>>().join(", ");
                println!("Resolving tie between: {}", candidates.bright_cyan());
            },
            _ => (),
        }
    }
}

/// Displays the winner.
pub fn winner(winner : Option<usize>, candidates : &Candidates) {
    match winner {
        Some(winner) => println!("{} {}", "Winner:".bright_blue(), candidates.get(winner).unwrap()),
        None => println!("{}", "The election was a tie".bright_blue()),
    }
}

/// Notifies the user if the threshold was adjusted.
pub fn threshold_squash(prev_threshold : f64) {
    if prev_threshold < 0.0 {
        println!("{} Threshold was below the allowed range, and set to 0", "Warning:".yellow().bold())
    }
    else if prev_threshold > 1.0 {
        println!("{} Threshold was above the allowed range, and set to 1", "Warning:".yellow().bold())
    }
}

/// Displays a CSV error.
pub fn csv_error(error : csv::Error) {
    println!("{} {}", "CSV Error:".red().bold(), error);
}
