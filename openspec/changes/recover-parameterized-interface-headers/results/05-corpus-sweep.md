# 任务 2.5 / 3.2：corpus 双腿扫描（渲染差分）

```sh
python3 results/05-corpus-sweep.py > results/05-corpus-sweep.out
```

- 语料：`tests/fixtures` 全部 **735** 个 `.class`（与 `jarde-reader` 的 fixture 普查同数）。
- 两条腿：**loose**（每类 `single_class` 策略单独渲染）、**family**（每个含 class 的目录打成一个 jar，
  条目名取**各 class 文件自身的 internal name**，逐类 `plain_jar` 渲染为根类——只有这条腿里 jar 内自带的
  接口定义与 `$` 名父类可被证明）。
- 两个二进制：base = 本片改动前（`37027c690fe634149a7346749f3b0b0f7a89e21b06f1ff44acc5bf922d0bb247`）；
  patched = 本片改动后（sha256 见 `results/06-gates.md`）。
- 自检（先于任何计数）：渲染必须带 `// jarde: presentation of` 自述头（否则记 `ERROR-RENDER`，绝不计为"相同"）；
  正例 `BR$Impl` 在 family 腿必须由裸头变参数化；负例 `BareBox` 必须不动。

## 结果

| 腿 | 类数 | 移动 |
| --- | --- | --- |
| loose（single_class） | 735 | **0** |
| family（plain_jar） | 735 | **12** |

family 腿的 12 个移动类**恰为**普查预期集：

| 移动类 | 归属 | 变化 |
| --- | --- | --- |
| `p3-bridge-projection/br-family/v8/BR$Impl.class` | 普查 4 类（接口腿） | 裸头 → `implements java.lang.Comparable<BR$Impl>`，`compareTo(Object)` 桥 hidden |
| `p3-bridge-projection/positive/v8/BridgeProbe.class` | 普查 4 类（接口腿） | 裸头 → `implements BridgeApi<java.lang.String>`（协变桥本已 hidden） |
| `p3-bridge-projection/br-family/v8/BR$StrBox.class` | 普查 4 类（父类腿） | `extends BR$Box` → `extends BR$Box<java.lang.String>`，`set` 桥 hidden |
| `p3-bridge-projection/bridge-superclass-precondition/v8/Spec.class` | 普查 4 类（父类腿） | `extends Outer$Box` → `extends Outer$Box<java.lang.String>`，`set` 桥 hidden |
| `p3-interface-header-projection/{v8,v8-javac8}/IfaceImpl.class` | 本片新 fixture（正例） | 同上，参数化头 + 桥 hidden |
| `p3-interface-header-projection/{v8,v8-javac8}/MultiIface.class` | 本片新 fixture（多接口部分可证） | 参数化项投影、`Runnable` 保持 |
| `p3-interface-header-projection/{v8,v8-javac8}/ErasedCall.class` | 本片新 fixture（实参边界格） | 参数化头 + 桥 hidden（body 实参仍擦除拼写，见 03-fixtures） |
| `p3-interface-header-projection/{v8,v8-javac8}/Arity.class` | 本片新 fixture（arity 矛盾格） | 新增 header 拒绝 |

**越界项为零**：除上述 12 个（= 普查 4 + 本片新增 fixture 的 8 个预期格）之外没有任何类移动。
`Unresolved`/`TypeUse`/`BareBox`/`NestedExtends`/`ArityExtends`/`Multiseg` 与全部既有语料**逐字未变**。

## 首轮（无效）与它的发现

首轮脚本（shell + `zip -j`）与首版实现一起跑出 30 个移动类，其中第 5 个既有类 `AC.class`
（`recover-platform-interface-argument-widening` 的 fixture，`implements java.util.Comparator<String>`）
暴露了一个真实缺陷：**当接口一个都没投影时，实现仍会发布"投影声明"**（文本只多一行
`// jarde: class Signature ... projected after parent erasure proof`），使 `AC` 这类类的呈现变化。
修法：接口路径在"没有任何条目带实参"时返回 `Ok(None)`（什么都不发布 = 改动前状态）。转录见
`results/05-corpus-sweep-first-run-invalid.out`（含 `AC` 的 diff 与 220 条 `ERROR-RENDER`——后者是
`zip -j` 把带包路径的类压平造成的**脚手架**产物，不是语料差异）。

## 覆盖限制（如实记录，两个二进制同待遇）

57 个 `ERROR-RENDER`（base 与 patched **都**失败，故不构成差分、也不隐藏差分）：

- 56 个 family 腿：同一目录里有**多个文件声明同一个 internal name**（本仓按设计提交的字节补丁探针族，
  如 `WA-*.class`、`em01/Shape-I-*.class`、`Loc.punct.class`），一个 jar 无法同时容纳两个同名类，最后写入者
  胜出，其余文件按自己的名字查不到 ⇒ 渲染失败。这些探针在 **loose 腿逐文件覆盖** ✓。
- 1 个 loose 腿：`proved-java-structure/package-info-basic/v8/p/package-info.class`（standalone
  `package-info` 无类可呈现；该类在测试里以 jar 环境呈现，见 `tests/class_source.rs` 的 package-info 用例）。
