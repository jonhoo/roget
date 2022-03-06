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
`wordle.txt` by grabbing the words from the Wordle source code (that's
also how you get `answers.txt`). Then, grab the ngram dataset by
downloading [these][1grams], then running.

```
rg -Iz "^[a-zA-Z]{5}_[A-Z]+\t" 1-*-of-00024.gz > 5-letters.txt
awk -F'\t' '{print $1"\t"$NF}' 5-letters.txt | sed 's/_/,/' | awk -F, '{print $1" "$(NF-1)}' > 5-letters-occur.txt
tr A-Z a-z < 5-letters-occur.txt | sort > 5-letters-lc-sorted.txt
awk 'BEGIN {w="";v=0} {if (!w) w=$1; else if ($1==w) v=v+$2; else { print w" "v; w=$1; v=$2; } } END {print w" "v}' < 5-letters-lc-sorted.txt > 5-letters-lc-sorted-combined.txt
join -a 1 wordle.txt 5-letters-lc-sorted-combined.txt | sed 's/\([a-z]\)$/\1 1/' > dictionary.txt
```

[1grams]: https://storage.googleapis.com/books/ngrams/books/20200217/eng/eng-1-ngrams_exports.html
