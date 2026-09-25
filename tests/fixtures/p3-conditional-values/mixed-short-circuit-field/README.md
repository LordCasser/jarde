# Mixed-polarity short-circuit field values

`MixedBooleanField.andOr(Z)V` stores `(a && b()) || c()` and `orAnd(Z)V` stores `(a || b()) && c()`. `b()` and `c()` count invocations independently. The frozen class was built with `javac --release 8 -g:none`; source SHA-256: `8bd0cf80ec39a2c24a6bd7eba4c85059ca2a4b5171bc4718ffe1584118b679c7`, class SHA-256: `13deab71668ec6c26f1a636f5a646bc87156d9aa46444a5b1a1eac9896bcbe5f`.

Compile `MixedBooleanField.java` and `Runner.java` with `javac --release 8 -g:none`, compare the resulting `MixedBooleanField.class` byte for byte with the frozen class, and run `java -Xverify:all Runner`. The runner prints both methods for all eight combinations of `a`, `bValue`, and `cValue` as `method:mask:result:bCalls:cCalls` (`method` 0 is `andOr`, 1 is `orAnd`). The 16-line reference trace and three-way comparison are in [analysis.md](../../../../openspec/evidence/java-syntax-2026-09-25/mixed-short-circuit-field/analysis.md).
