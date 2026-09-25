# Narrow integer return fixture

`NarrowIntegerReturns.class` is the only permanent class. Its source deliberately compiles all
returns as `int`, then an exact constant-pool descriptor patch changes selected method returns to
`byte`, `char`, or `short` without changing any Code bytes. The class has byte/char/short local
readback methods plus an int-return control, direct returns, field post/pre increments, and
synchronized returns. The runner records out-of-range values, updated int fields, and the null
monitor exception; the helpers and runner remain source-only.
