# Enum field alias boundary

Run `python3 replay.py /tmp/jarde-enum-alias-replay` from this directory. The script copies the frozen swapped-map input and replaces only the second enum constructor expression in `Hue.<clinit>` with `getstatic Hue.RED` plus seven `nop` bytes. The `BLUE` field then aliases `RED` while the class remains accepted by `java -Xverify:all`.

The original helper writes key 2 for `RED` and then key 1 for `BLUE`; because both fields now refer to the same ordinal, the latter write wins. Original output is `1|1, 1|1, 3|3, null|0`. A direct `switch (hue)` with `case BLUE → mark(1)` followed by `case RED → mark(2)` compiles under `javac --release 8`, but its newly generated helper writes the cases in source order and yields `2|2, 2|2, 3|3, null|0`.

Thus a proven helper map and `ACC_ENUM` fields alone do not justify projection. The enum field objects and ordinals must also be proved distinct, or this input must be refused. `replay.py` fixes the before/after class SHA-256 values and emits `summary.json` plus `javap` output in the chosen directory. It does not require a Jarde build.
