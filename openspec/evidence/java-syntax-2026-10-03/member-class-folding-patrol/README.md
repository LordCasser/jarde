# 成员类折叠巡查（2026-10-03）——大颗粒里程碑立案

[嵌套拼写巡查](../2026-10-02/nested-name-spelling-patrol/README.md) 遗留的定向取证（主线 `ea791746`）。固定转录 [fixture](fixture/)（M1 多子家族 fam.jar / M2 单子 solo.jar / M1User 跨类头部引用；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`hi`/`ok`）、o2.out（`7`）。

## 现状矩阵

| 场景 | 主线 Jarde |
| --- | --- |
| 外围类文本（jar 输入） | 各自独立恢复，方法体/信封内 `M1$Deep`/`M1$Err`/`M1$Solo` 池 `$` 直拼 |
| **多直接静态成员（M1：Base/Ctrl/Deep/Err/Inner 五子）** | `member_family.state="refused"`，reason="multiple direct static member rows are outside the one-child family subset" |
| **单直接静态成员（M2：Solo）** | one-child 扫描通过但**不折叠**——输出文本无 `static class Solo` 声明（one-child 通道的消费面是匿名/枚举投影对〔`DeclarationPair` 0x0609/0x0409〕，非成员类声明折叠） |
| 嵌套类各自单类输出 | 完整恢复（类头 `extends M1$Base implements M1$Ctrl` 池拼写——家族平铺惯例下可编） |

## 定性

**成员类折叠（"呈现嵌套声明"能力）缺失**是枚举折叠（已做）之后的对称大颗粒：枚举家族把常量+匿名子类折叠进外围文本并重写名字；普通静态成员类/接口无对应通道。它一并解决 nn 片登记的全部遗留位——折叠作用域内类头/字段/throws/体内引用自动经折叠投影获得源码拼写（枚举折叠的名字重写机械先例）。非静态成员类另含 `this$0` ctor（synthetic-ctor 片已处理序）。

## 处置方向（MVP 分层）

- **`recover-member-class-static-folding`（先，本片）**：直接静态成员类/接口（任意数量，Java 源码可写的 `static class/interface`）折叠进外围文本——每个子类完整文本经既有恢复通道产出后按 InnerClasses 行序嵌入，名字重写复用枚举折叠投影机械；折叠作用域内引用（含外围类头/字段/throws/体内与子类间互引）按源码拼写重写；孙代（`Inner$Leaf`）MVP 不折叠（子类文本内池拼写，登记）。M1/M2 家族整 jar 重编行为一致。
- **`recover-member-class-inner-folding`（后）**：非静态成员类（this$0 序、外部实例捕获）。

原 class 为行为基准。
