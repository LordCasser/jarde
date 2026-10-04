# 接口 default/static 方法巡查（2026-10-04，root）——验证健康，非缺口

按 Goal "持续巡查各个反编译语法分析模块，构造各种 Java 语法场景，编译，然后对比源码、jadx、jarde" 巡查 Java 8 头号特性：**接口 `default`/`static` 方法及其方法体**。主线 HEAD 二进制（`559b9fea`）。

**结论：四形全部正确恢复、可重编、行为逐字一致。非缺口——本记录的目的是避免将来重复巡查。** inventory 账本无对应单元（`declarations-types.md` 的 DT-05 是匿名接口实现、DT-15 是泛型形参、DT-22 是注解默认值、DT-29 是接口强转，均不覆盖接口 default/static 方法体），故此前无人验证过这一形。

## 探针（[fixture/I1.java](fixture/I1.java)，`javac --release 8`）

一个接口含四种形态，覆盖不同的体复杂度：

```java
interface Greeter {
    String greet(String who);                                        // 抽象
    default String hello() { return greet("world"); }                // default 调用抽象方法
    default int count(String s) { int n=0; for(char c: s.toCharArray()) if(c=='a') n++; return n; }  // default 带增强 for + 计数
    static Greeter upper() { return who -> who.toUpperCase(); }      // static 返回 lambda
    default String compose() { return "x" + hello() + count("banana"); }  // default 拼接 + 调用两个 default
}
```

原类运行基线（[results/original.out](results/original.out)）：`hi world` / `3` / `BOB` / `xhi world3`。

## jarde 呈现（[results/Greeter.txt](results/Greeter.txt)）

`class-source --input fam.jar --class 'I1$Greeter'`：**quotes=0、not recovered=0**，四形全部结构化恢复：

| 形 | 呈现 |
| --- | --- |
| 抽象 | `public abstract java.lang.String greet(java.lang.String arg1);` |
| default 调用抽象方法 | `public default java.lang.String hello() { return this.greet("world"); }` |
| default 带增强 for + 计数 | 完整 `for (char local6 : local3) { if (local6 == 97) { local2 = local2 + 1; } }` + `return local2;` |
| static 返回 lambda | `public static I1$Greeter upper() { return (java.lang.String arg0) -> arg0.toUpperCase(); }` |
| default 拼接 + 双调用 | `return "x" + this.hello() + this.count("banana");` |

## 三方验证（root 实测）

- **重编**：渲染的接口单元 `javac --release 8 -cp ..` → **exit 0**。
- **运行**：重编接口 + 物理 `I1` 入口，`java -Xverify:all` → `hi world` / `3` / `BOB` / `xhi world3`，**与原类四行逐字一致**。

即恢复文本不仅无引注，且经编译与验证运行证实语义等价。

## 备注（非缺口，但记录两点观察）

1. 增强 for 呈现为 `for (char local6 : local3)`（先 `local3 = arg1.toCharArray();` 再遍历），是把 `toCharArray()` 结果提取为局部——忠实且可编译，非语义偏差。
2. `if (local6 == 97)` 呈现为整数比较而非 `== 'a'` 字符字面量（对应 `present-proved-java-structure` 未勾项 2c.30 的 char 收窄拼写域，已由 `emit::char_literal` 覆盖其它位置）。这是**呈现润色**级别，不影响可编译性与行为，且已有对应账本条目，故不在本巡查另立项。

原 class 为行为基准。
