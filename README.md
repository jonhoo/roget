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
