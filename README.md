# Swyss-rs

A Rust library to manage tournaments run with the
[Swiss tournament system](https://en.wikipedia.org/wiki/Swiss-system_tournament).
Includes a basic client that can read participants from a text file,
presents round-wise pairings sequentially in a random order,
accepts score inputs,
and prints the standings after the tournament finishes.
Also supports running a "tournament" between all image files in a given directory by displaying the paired images using `feh`
(use this to e.g. select the best candidate from among a list of photos).

## Getting Started

Create a plain text file holding the player names,
separated by linebreak.
Then:

```
$ swyss players.txt
```

to run a simple tournament.
Swyss will automatically create pairings in each round based on standings according to the standard Swiss rules.
Pairings are presented sequentially in a random order on the command line:

```
PAIRING:
[1] Player 1
[2] Player 2
```

Individual player scores can then be entered one after the other in the same order:

```
[1] Player 1 > 2
[2] Player 2 > 1
```

Possible results are 2-0,
0-2,
2-1,
1-2,
and 1-1 (draw).
All other inputs are rejected and the same pairing is prompted again.

The number of rounds is calculated according to the minimum number of rounds necessary to rank players sufficiently,
which is typically thought to be `ceil(log_2(num_players))`.

If the number of players is uneven,
a bye will be awarded each round to the lowest-ranked player that has not yet received a bye.

Final standings are printed including the tiebreakers _match points_,
_opponents' match win percentage_,
_game win percentage_,
and _opponents' game win percentage_,
applied in that order.
If all tiebreakers are equal,
the tie is broken at random.

### Prerequisites

Rust; `feh` if you want to compare images.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/jpfender/swyss-rs/tags).

## Authors

- **Jakob Pfender** - _Initial work_ - [jpfender](https://github.com/jpfender)

<!--See also the list of [contributors](https://github.com/jpfender/swyss-rs/contributors) who participated in this project.-->

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details
