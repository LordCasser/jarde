# 单出口循环臂的局部汇合测试

正例源码沿用 `openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/input/join/cf08join/LoopIfJoin.java`。负例源码在本目录，分别加入额外 `break`、`return` 与异常边。两个 class 均由 `javac --release 8 -g:none` 编译；测试中的另外两个负例只改变正例的一个分支目标，保留 Java 8 可验证的 StackMap 目标。

```sh
javac --release 8 -g:none -d tests/fixtures/p3-loop-arm-join/v8 openspec/evidence/java-syntax-2026-09-27/cf08-endless-loops/input/join/cf08join/LoopIfJoin.java tests/fixtures/p3-loop-arm-join/LoopIfJoinExtraBreak.java
```

`LoopIfJoin.class` SHA-256：`542c856a156103f8c4e20372fcbeb3bc767b0c706f3270d73d5ec6dddfefdc2a`。
`LoopIfJoinExtraBreak.class` SHA-256：`14ba9f16f39c051b680a4d886656571e40f41a8b13a881ba6f0ddef6ebe6d954`。
