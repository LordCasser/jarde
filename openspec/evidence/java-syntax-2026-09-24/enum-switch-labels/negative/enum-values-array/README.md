# 枚举数组路径反例

`replay.py OUTPUT_DIRECTORY` 从冻结的 Java 8 `Hue.class` 精确替换两段等长 Code 字节，不改变声明、方法签名、字段或常量初始化。输出目录中的每个变体均由 `java -Xverify:all` 实际执行，`summary.json` 记录结果；同次把 `Hue.java` 和 `EnumSwitchSubject.java` 编译为直接枚举源码，显示错误投影会改变可观察结果。

| 变体 | `Hue.class` SHA-256 | 已验证原类输出 | 直接源码输出 |
| --- | --- | --- | --- |
| 私有数组工厂将第 2 号元素写成 `null` | `6e618e0562df89f88bddafc720d466bff48b96da8ffa7b9d5f227ae5300a36bc` | `RED,BLUE,null`；`1,2,3` | `RED,BLUE,GREEN`；`1,2,3` |
| 公开 `values()` 直接返回 `null` | `f3fa2fc27d29c125ec01b3e84264216dfb2070c57116360d13f2fb1f6185c23d` | `null`；`ExceptionInInitializerError` | `RED,BLUE,GREEN`；`1,2,3` |

固定 class 文件供 `class_initializer_candidates` 读取，固定 jar 供完整类 `class_source` 测试读取；重放脚本校验原 `Hue.class` 哈希及补丁字节唯一性。前者必须被工厂逐项装入证明拒绝，后者必须被公开 `values()` 的完整路径证明拒绝。工厂目标由 `<clinit>` 的真实 Callref 选出，不依赖 `$values` 或 `$VALUES` 的命名启发式。
