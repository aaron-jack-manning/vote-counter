# Vote Counter

This is just a program that I wrote to count instant runoff votes, according to a certain specification.

## Usage

It's usage is as follows:

```
vote-counter <CSV_PATH> <THRESHOLD>
```

The first command line arguments is the path to the `.csv` file containing the ballot papers, and the second is the threshold required to win, where 0.5 corresponds to a simple majority.

## Ballot File

Here is a sample ballot file:

| Peter | Mia | Hannah | Lee |
| ----- | --- | ------ | --- |
| 1     | 2   |        | 3   |
| 2     | 4   | 3      | 1   |
|       |     | 1      |     |

Each row represents a ballot paper, where preferenced are expressed starting at 1, and continuing until the voter no longer has a preference.

## Invalid Votes

The following votes are considered invalid:

- Multiple occurances of the same preference, for example:

| Peter | Mia | Hannah | Lee |
| ----- | --- | ------ | --- |
| 1     | 1   |        | 3   |

- A preference number which exceeds the number of candidates, for example:

| Peter | Mia | Hannah | Lee |
| ----- | --- | ------ | --- |
| 3     | 2   | 1      | 5   |

To be warned about invalid ballots, run in debug mode.

The code has been internally documented reasonably thoroughly so if you want to fork the repo and change the logic surrounding invalid votes I hope I have made that reasonably easy.

## Permitted Votes

Aside from the obviously valid votes, which number candidates 1 to a given preference as far as the voter may chose, votes which skip a preference are also considered valid, for example:

| Peter | Mia | Hannah | Lee |
| ----- | --- | ------ | --- |
|       |     | 3      | 1   |

where preferences are shuffled down such that in the above example, Hannah is considered to be the second preference.

A sample ballots file called `sample.csv` is provided which includes only valid ballots.

