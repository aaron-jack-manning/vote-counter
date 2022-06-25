mod ballot_box;
mod reporting;
mod candidates;
mod ballot;

use ballot_box::BallotBox;
use ballot_box::CountStatus::*;

use std::path;
use std::process;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version)]
/// Stores the command line arguments.
struct Args {
    /// Path to the CSV containing the ballots.
    #[clap()]
    path : path::PathBuf,

    /// Threshold to win.
    #[clap(long, short, default_value = "0.5")]
    threshold : f64,

    /// Generate report of counting.
    #[clap(long, takes_value = false)]
    report : bool,
}

/// Primary entry point to vote counting algorithms.
fn count(args : Args) -> Result<(), csv::Error> {

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
            println!("An error occured reading the CSV data: {}", error);
            process::exit(exitcode::DATAERR);
        }
    }
}
