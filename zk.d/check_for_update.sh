#!/bin/sh

if echo "$1" | grep -q '_zettel.json'; then
    echo 'skipping _zettel.json' 1>&2
    exit 0
fi

start=$(grep -n -m 1 '^---$' "$1" | cut -d : -f 1)
end=$(grep -n -m 2 '^---$' "$1" | tail -n 1 | cut -d : -f 1)
tail -n "+$(($start + 1))" <"$1" | head -n "+$(($end - $start - 1))"
