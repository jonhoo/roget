[![codecov](https://codecov.io/gh/jonhoo/roget/branch/main/graph/badge.svg?token=BMvVKvRuYO)](https://codecov.io/gh/jonhoo/roget)

Original version live-coded [on YouTube](https://youtu.be/doFowk4xj7Q).

The implemented algorithm is almost exactly what was outlined (and
_very_ well explained) in [this 3blue1brown video][3b1b].

Please do tinker with it and see how much you can push it — there's
almost certainly gains to be had! I've also left some TODOs from the
3b1b algorithm that should improve the guesses a fair bit. It'd also be
really neat to add in a mode for computing the _first_ word by computing
multiple levels of expected information (again, like 3b1b), instead of
just hard-coding it like we do at the moment.

[3b1b]: https://www.youtube.com/watch?v=v68zYyaEmEA

# Dataset

If you want to remake `dictionary.txt` yourself, first, make
`corpus/wordle.txt` by grabbing the words from the Wordle source code
(that's also how you get `answers.txt`). Then, grab the ngram dataset by
downloading [these][1grams]. Then run:

```bash
cd corpus
cargo r --release /path/to/1-*-of-00024.gz | tee ../dictionary.txt
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[1grams]: https://storage.googleapis.com/books/ngrams/books/20200217/eng/eng-1-ngrams_exports.html
