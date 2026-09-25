# Catch parameter escape boundary

`CatchEscape.java` was compiled with `javac --release 8 -g:none -Xlint:-options`.
The frozen `v8/CatchEscape.class` SHA-256 is
`2a4593c904beb9e5c415f87718bae977728a957badc7ad143a0088e4816a24ed`.

The regression changes the unique code sequence `4d 2c 4c 2b b0`
(`astore_2; aload_2; astore_1; aload_1; areturn`) to `4c 2b 4c 2b b0`.
The handler then stores its caught reference in slot 1, and the return after
the clause reads that same value. The patched class SHA-256 is
`c2d2a05944fce7fdaca200800191df30754f7d3de5ec32a6a6aad1646c3726fd`.
`java -Xverify:all` accepts the patched class when a driver calls `read`.
Its source cannot be reconstructed by treating the handler parameter and
the return local as one Java lexical binding.
