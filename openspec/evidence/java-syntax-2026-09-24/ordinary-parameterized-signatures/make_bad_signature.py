#!/usr/bin/env python3
import pathlib, sys
source, destination = map(pathlib.Path, sys.argv[1:])
data = source.read_bytes()
old, new = b'Ljava/util/List<', b'Ljava/util/Tree<'
assert len(old) == len(new)
assert data.count(old) == 9, data.count(old)
destination.write_bytes(data.replace(old, new))
