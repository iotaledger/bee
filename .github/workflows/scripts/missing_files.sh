#!/usr/bin/env bash

res=0
excluded_crates=($1)

exclude_crate () {
  local array="$1[@]"
  local seeking=$2
  local in=1
  for element in "${!array}"; do
    if [[ "./$element" == "$seeking" ]]; then
        in=0
        break
    fi
  done
  return $in
}

while read file
do
  dir=$(dirname "$file")

  if exclude_crate excluded_crates $dir; then
    continue;
  fi

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
