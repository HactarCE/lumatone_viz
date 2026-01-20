#!/usr/bin/env python3

import sys
import re

OCTAVE_OFFSET = 0

key_chan_regex = re.compile(r'Key_(\d+)=(\d+)\nChan_\1+=(\d+)')

seen = set()

def force_unique_key_chan(m):
    [id, key, chan] = map(int, m.groups())
    while (key, chan) in seen:
        chan += 1
        key -= OCTAVE_OFFSET
    seen.add((key, chan))
    return f'Key_{id}={key}\nChan_{id}={chan}'

with open('out.ltn', 'w') as out_file:
    with open(sys.argv[1]) as in_file:
        out_file.write(key_chan_regex.sub(force_unique_key_chan, in_file.read()))
