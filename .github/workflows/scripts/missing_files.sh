#!/usr/bin/env bash

res=0

while read file
do
  dir=$(dirname "$file")

  file="$dir/CHANGELOG.md"
  if [[ "$dir" != "." && ! -f $file ]]; then
    echo "Missing \"$file\" file."
    res=1
  fi

  file="$dir/LICENSE"
  if [ ! -f $file ]; then
    echo "Missing \"$file\" file."
    res=1
  fi

  file="$dir/README.md"
  if [ ! -f $file ]; then
    echo "Missing \"$file\" file."
    res=1
  fi

done <<< "$(find . -name "target" -prune -o -type f -name "Cargo.toml" -print)"

exit $res
