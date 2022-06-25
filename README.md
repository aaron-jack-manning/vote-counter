# Vote Counter 

An opinionated single transferrable vote counter for the command line.

## Installation

Installation can be done via cargo by running:

```
cargo install vote-counter
```

## Demo

A sample `csv` file in the appropriate format is provided at the top level of this repository. Run:

```
vote-counter sample.csv --report
```

to count the votes from that file.

## Arguments

Running `vote-counter --help` will output the following:

```
USAGE:
    vote-counter [OPTIONS] <PATH>

ARGS:
    <PATH>    Path to the CSV containing the ballots

OPTIONS:
    -h, --help                     Print help information
        --report                   Generate report of counting
    -t, --threshold <THRESHOLD>    Threshold to win [default: 0.5]
    -V, --version                  Print version information
```

explaining each argument and how to use it.

## Ballot File

The ballot file should be a `csv` formatted as below:

| Peter | Mia | Hannah | Lee | Fred | Julia |
| ----- | --- | ------ | --- | ---- | ----- |
|       | 2   | 1      |     |      | 3     |
| 1     |     | 2      | 3   | 4    |       |
| 5     | 4   | 3      | 1   | 2    | 6     |

Each row represents a ballot paper, where preferenced are expressed starting at 1, and continuing until the voter no longer has a preference.

## Validity of Votes

This program is generally permissive in the votes that are considered valid. If a ballot includes any number of non-negative preference numbers, none of which are repeating, the ballot is valid.

An invalid ballot occurs when the same preference is expressed twice.

For example, the following are not valid:

| Peter | Mia | Hannah | Lee | Fred | Julia |
| ----- | --- | ------ | --- | ---- | ----- |
| 1     | 1   |        | 3   |      |       |
| 0     | 1   |        | 4   |      | 4     |
| 2     |     | 2      |     | 1    |       |

However the following are valid:

| Peter | Mia | Hannah | Lee | Fred | Julia |
| ----- | --- | ------ | --- | ---- | ----- |
| 0     | 1   | 2      |     |      | 3     |
|       |     | 1      | 4   |      |       |
| 2     |     | 5      |     |      | 1     |

Negative numbers are simply ignored.
