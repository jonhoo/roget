# Original version live-coded [on YouTube](https://youtu.be/doFowk4xj7Q)

The implemented algorithm is almost exactly what was outlined (and
_very_ well explained) in [this 3blue1brown video][3b1b].

Please do tinker with it and see how much you can push it — there's
almost certainly gains to be had! I've also left some TODOs from the
3b1b algorithm that should improve the guesses a fair bit. It'd also be
really neat to add in a mode for computing the _first_ word by computing
multiple levels of expected information (again, like 3b1b), instead of
just hard-coding it like we do at the moment.

[3b1b]: https://www.youtube.com/watch?v=v68zYyaEmEA

## Dataset

If you want to remake `dictionary.txt` yourself, first, make the
`wordle-answsers.json` and `wordle-valids.json` by grabbing the words from the Wordle source code
(as of 2022-03-06: around line 1135 after using the devtools formating).

If you have `cargo-make`, `ripgrep`, `sd`, `jd` installed you can just run:

```bash
cargo make corpus
```

It will dowload the [1gram google dataset][1gram] and create a `dictionary.txt` file.

[1grams]: https://storage.googleapis.com/books/ngrams/books/20200217/eng/eng-1-ngrams_exports.html
