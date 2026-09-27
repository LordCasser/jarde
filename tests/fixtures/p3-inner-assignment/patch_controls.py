#!/usr/bin/env python3
"""Make verifier-valid near misses from the fixed CF-06 Java 8 class."""

from pathlib import Path
import sys


original = Path(sys.argv[1]).read_bytes()
output = Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=True)

# lengthBranch bytecode: aload_0; invokevirtual isEmpty; ifne; aload_0;
# invokevirtual length; dup; istore_1; iconst_5; if_icmple.
window = bytes.fromhex("2a b6 00 07 9a 00 0d 2a b6 00 0d 59 3c 08 a4")
assert original.count(window) == 1

# `dup` at BCI 13 consumes the surviving first copy and makes two new test copies.
# The original copy is no longer consumed directly by the comparison.
extra = original.replace(window, window[:-2] + bytes.fromhex("59 a4"), 1)
(output / "ExtraCopy.class").write_bytes(extra)

# Swap the same-width method reference to String.isEmpty:()Z. JVM boolean and int are
# both category-1, so the class verifies, but the local's descriptor-derived Java type
# is boolean and it cannot be compared with the integer literal 5.
wrong = original.replace(window, window[:9] + bytes.fromhex("00 07") + window[11:], 1)
(output / "WrongType.class").write_bytes(wrong)
