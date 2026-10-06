# `recover-comparable-argument-widening` 的冻结 fixture

`CO`/`COX` 的源与两条 javac 腿的 class 文件。腿：

```sh
javac --release 8 -Xlint:-options -d v8 *.java                       # javac 23.0.1
/Users/lordcasser/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -d v8-javac8 *.java
```

## 锚（`CO`，两条腿逐字一致）

- `callGen`：`max("a","b")` —— `String → java.lang.Comparable`（递归泛型巡查的调用点）；
- `callGen2`：`max(1,2)` —— **装箱参与**（`Integer.valueOf` 后入表判定，八装箱行）；
- `same`：同一调用的参数形（非字面量实参）；
- `main`：`RG$IntNode.cmp` 形（`val.compareTo(o.val)` 的擦除字段 cast + `compareTo` 恢复）——
  巡查判定的健康面，逐字不变。

## 负例（`COX`，仍拒）

`big`：`java.math.BigInteger → java.lang.Comparable` —— `BigInteger` 事实上实现 `Comparable`，
但 change 严格限定 java.lang 九行（String + 八装箱），`java.math` 及其余 JDK 实现者不入表。

## class 文件 SHA-256

| 文件 | sha256 |
| --- | --- |
| `v8/CO.class` | `142ad2918e7e8a4ec19e063dbc858691a9e794791399f02f59a51411d99b41c3` |
| `v8/CO$Node.class` | `d0001b809bd87e853673a8bd7b709d4445653130a6fa863f83e825d077ca865d` |
| `v8/CO$IntNode.class` | `1e52e026193b491317c1790f9766bfc0faad9a1a04f83ad9cb0a88b0bc7b6a37` |
| `v8/COX.class` | `f1f208961429db94bd3b55d5878b13272a35dbf06c7472b304eb7d182dc07194` |
| `v8-javac8/CO.class` | `daabd0650c7694d2657cfc527c1970b99d1b5962520ea98d29e69fcb86b694b2` |
| `v8-javac8/CO$Node.class` | `5e36f2e0cd2825dfb62584553e5138b62d43239342902836d5eda7f1e8f91691` |
| `v8-javac8/CO$IntNode.class` | `3bb059cf6ea59567cd4f25f76e1d6643b12bd8b4103ef8eb5852a6a73d3c8e30` |
| `v8-javac8/COX.class` | `57cfecee284c509db52fd439e53837b7874874f89ef554d166e5461299a9ca5e` |

行为（replay 与 fixture 自身一致）：`1/b/2/b`。
