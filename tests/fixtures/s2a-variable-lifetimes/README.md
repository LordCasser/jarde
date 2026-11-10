# S2a variable lifetime inputs

The four committed jars are the frozen `DevCases` class-source inputs from
`openspec/changes/restructure-demand-driven-recovery/results/s2a-variable-legalization-v1/build/`.
Each archive contains the same `DevCases` family and `DevDriver`; the four variants cover
Corretto 8 and OpenJDK 23 compilation, with and without debug metadata.

| File | SHA-256 |
| --- | --- |
| `corretto8-debug.jar` | `9af4ac06a4f3fe61a5b6cfa4f45ac61a61ff01b76f730833b5452ec8a1bb5b69` |
| `corretto8-no-debug.jar` | `7da1b760c6b6144d62c977fb9a0bbff585c0e13562a27aed67c99a5fea9b042a` |
| `openjdk23-debug.jar` | `b55373e05f096a1f70220d7395f19bc7a3f8ba729dea11adf67b996d0da51058` |
| `openjdk23-no-debug.jar` | `83a3fb5cb8304d1355bfaacadb577b37d6306b2942a221bd957a09ea0e12a913` |

`DevCases.java` is the frozen source used to produce the jars (SHA-256
`969dbf3b74cbb6b9222be31301a7a44516551916a5dee200cafb01368d242ab6`). `DevDriver-r2.java` is
the separately corrected runtime oracle driver (SHA-256
`37fcf4c05cf1e927fbde1f6241520d50016e926f821b67657290e056f2713dd9`). The original driver
remains in the change results; R2 changes only the `swapAcrossBackedge(2, 5, 3)` expected result
from 413 to 418, as independently confirmed by the eight frozen runtime traces. The Rust regression
uses only the jars and does not compile or execute either Java source.
