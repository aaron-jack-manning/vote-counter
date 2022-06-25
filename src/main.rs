mod ballot_box;
mod reporting;
mod candidates;
mod ballot;

use ballot_box::BallotBox;
use ballot_box::CountStatus::*;

use std::path;
use std::process;

use clap::Parser;

/// Adjusts threshold to be within permitted range, warning the user.
fn adjust_threshold(threshold : f64) -> f64 {
    reporting::threshold_squash(threshold);
    if threshold < 0.0 {
        0.0
    }
    else if threshold > 1.0 {
        1.0
    }
    else {
        threshold
    }
}

#[derive(Parser, Debug)]
#[clap(author, about, version)]
struct Args {
    /// Path to the CSV containing the ballots.
    #[clap()]
    path : path::PathBuf,

    /// Threshold to win (from 0.0 to 1.0).
    #[clap(long, short, default_value = "0.5")]
    threshold : f64,

    /// Generate report of counting.
    #[clap(long, takes_value = false)]
    report : bool,
}

/// Primary entry point to vote counting algorithms.
fn count(mut args : Args) -> Result<(), csv::Error> {

    args.threshold = adjust_threshold(args.threshold);

    let mut ballot_box = BallotBox::from_file(&args.path, args.report)?;
    
    let winner = loop {
        match ballot_box.status(args.threshold, args.report) {
            Winner(winner) => break Some(winner),
            Tie => break None,
            Runoff(to_eliminated) => ballot_box.runoff(to_eliminated),
            Promotion(to_promote) => ballot_box.promote(to_promote),
        }
    };

    reporting::winner(winner, &ballot_box.candidates);
    
    Ok(())
}

fn main() {
    let args = Args::parse();

    match count(args) {
        Ok(_) => {
            process::exit(exitcode::OK);
        },
        Err(error) => {
            reporting::csv_error(error);
            process::exit(exitcode::DATAERR);
        }
    }
}
