#!/usr/bin/env bash

res=0

while read file
do
  dir=$(dirname "$file")
  license="$dir/LICENSE"
  if [ ! -f $license ]; then
    echo "Missing $license file!"
    res=1
  fi
done <<< "$(find . -name "target" -prune -o -type f -name "Cargo.toml" -print)"

exit $res
