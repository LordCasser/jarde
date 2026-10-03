# 非静态成员类折叠巡查（2026-10-03）——嵌套声明里程碑第二层

[成员类折叠里程碑](../member-class-folding-patrol/README.md) 的对称续片取证（主线 `74c6980e`）。固定转录 [fixture](fixture/)（N1：非静态 Inner〔捕获 `base`〕+ 静态 Stat 混合家族 + 限定 `outer.new Inner(9)` + 静态语境 `new N1().new Inner(3)`；SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)），行为基线 orig.out（`10`/`7`/`13`）。

## 现状矩阵

| 场景 | 主线 Jarde（jar 输入） |
| --- | --- |
| N1 外围文本 | 方法体恢复健康（`new N1$Inner(this, arg1)`、`new N1$Inner(new N1(), 3)`） |
| **非静态成员类折叠** | `member_family.state="prepared"` 但无嵌套声明——Inner/Stat 均未折叠（静态 Stat 混入非静态家族后整族走 prepared 分离路径，上片 `StaticMembers` 多子化只对纯静态族生效） |
| 构造呈现 | `new N1$Inner(this, arg1)` 池拼写 + **合成首参透传**（this$0 显式出现在调用位） |
| 合成访问桥 | `static int access$000(N1)` 完整呈现（源码无此成员——javac 对 `base` 私有捕获生成） |

## 缺口分层（真实源码 vs 现呈现）

1. **折叠缺失**：`class Inner { … }`（非静态）与混合族静态 `static class Stat` 均无嵌套声明。
2. **限定 new**：`outer.new Inner(9)`/`new N1().new Inner(3)` 应呈现限定形，现 `new N1$Inner(outer, 9)`（合成首参透传——行为对但非源语法）。
3. **this$0 字段与 ctor 首参**：折叠后应隐藏（synthetic-ctor 片已处理序；呈现层未消参）。
4. **access$000 桥**：折叠后应隐藏（synthetic accessor，javac 模式知识）。

## 处置方向（MVP 分层，按依赖序）

- **`recover-inner-class-static-mixed-folding`（先，已实现）**：混合族中**静态成员类**先行折叠（`StaticMembers` 判定从"纯静态族"扩为"族内静态子集"）——机械全复用上片，零新证明。实现与验收证据见 [`fold-mix/`](fold-mix/README.md)（分支取证、三变体前后、折叠输出 SHA、corpus 双腿扫描、N1 剩余唯一编译缺口=第二片限定 new）。
- **`recover-inner-class-instance-folding`（后，已实现）**：非静态成员类折叠——this$0 消参（ctor 首参与字段声明隐藏）、`outer.new` 限定呈现、access$000 消桥、捕获字段经桥引用还原为直接 `outer.base` 语境。实现与验收证据见 [`fold-inst/`](fold-inst/README.md)（三项取证、IV1-IV4 变体前后、N1 家族集 `10/7/13` 三方一致、corpus 双腿扫描 7 类变化全部为折叠目标形态、孙代链/写桥登记缺口）。

原 class 为行为基准。
