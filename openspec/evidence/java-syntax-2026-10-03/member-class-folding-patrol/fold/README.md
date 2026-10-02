# 成员类静态折叠实现证据（change `recover-member-class-static-folding`）

基线：worktree 主线 `4da1fe84`（实现前重放，fixture SHA 与
[`../results/fixture-sha256.txt`](../results/fixture-sha256.txt) 全部 12 项一致）。行为基线重放：
原 jar `java -Xverify:all` 下 M1=`hi`/`ok`、M2=`7`、`M1User.run()`=`hi`。

## 1. 取证重放（task 1.1）

- **多子拒绝位**（`src/member_inner.rs::scan_family_root`，主线 ~L1326-1345）：第二个静态行直接
  `Refused("multiple direct static member rows are outside the one-child family subset")`——M1 的
  Ctrl(0x0608) 甚至到不了接口门（行序 Deep→Base 先触发）。消费面确认：`prepare_class_source_member_family`
  仅准备一个 child，`project_class_source_member_family` 的窄证书要求 child 无字段、方法 0x0401
  全抽象、且全文本无第二处物理类型使用——M2 的 Solo 落在 `calls Refused("static declaration-only
  child is outside the abstract member slice")`，投影拒绝、无折叠。
- **枚举折叠名字重写机械**（`prove_nested_enum_method_texts`）：`java_code_name_spans` 词法扫描 +
  source-map 最小段锚定 + 指令 CP 验证 + `member_family_recovered_method_text_with_declaration`
  重排。本片复用全部四件；泛化点：(a) 指令验证从 getstatic+FieldRef 扩为"任一 CP 条目
  （Class/FieldRef/MethodRef owner）名指折叠类"，并接受异常表 catch 锚（handler_bci +
  catch_type）；(b) 锚定段从"最小段"放宽为"全部覆盖段"——合成 cast/receiver 限定符的 token 由
  包围表达式段锚定（StaticEchoNegative run() 的 `(X) make()` 形态即此）；(c) 声明重写门从
  descriptor 扩为 descriptor ∪ `Exceptions` 属性（throws 位唯一不在 descriptor 里的声明位）；
  (d) 字段声明经字段自身 descriptor 门 + 暂存文本通道（`source_text_with_member` 的
  `field_texts`）平移 derived。
- **孙代/嵌套行顺序**：折叠只收外围类自身 InnerClasses 的直接静态行；孙代行（outer=子类）不入列，
  其池拼写按 nn 片自嵌套规则原样保留（见 §5 登记缺口）。外围类自身带 outer 的 self 行（如
  `M1$Inner` 单类呈现）命中既有拒绝"selected root has an InnerClasses self row naming an outer
  class"，保持逐字分离——无需新门。

主线基线（`before/`，`-fold` 前文本）：M1 全家族 + M1User + M2 家族逐类输出均无嵌套声明；
member_family 状态：M1=`refused(multiple direct static member rows…)`、M2=`prepared`（capture
static_no_capture + calls refused）、`M1$Inner`/`M1$Deep`=`refused(self row…)`。

## 2. 变体冻结（task 1.2，`fixture-variants/FV*`）

| 变体 | 形态 | 原类 `-Xverify:all` 输出 | 实现前 member_family |
| --- | --- | --- | --- |
| FV1 VIface | 包私静态接口 `Mark`（0x0608）+ 实现类 | `5` | refused（"not a source-spellable named class"——接口门） |
| FV2 VSib | 子 extends 兄弟 + throws 兄弟异常 + 兄弟类型字段 | `3` | refused（multi-static） |
| FV3 VGrand | 孙代 `Inner$Leaf`（无折叠域内引用） | `6` | refused（multi-static） |
| FV4 VInner | 非静态成员类（this$0 捕获族） | `4` | prepared（capture 通道，calls refused） |
| FV5 VGRef | 孙代被折叠域内参数位引用 | `8` | refused（multi-static） |

各自原类 `java -Xverify:all` 通过（`*.orig.out`）。实现前后文本见 `before/FV*-before.txt` 与
`results/FV*-after.jarde.java`：FV1/FV2/FV3/FV5 折叠，FV4 逐字不变。

## 3. 折叠输出与重编（task 2.1）

`results/M1-fold.jarde.java`（五个嵌套声明：`static class Deep extends Base implements Ctrl`、
`static class Base`、`static class Inner`、`static class Err extends java.lang.Exception`、
`static interface Ctrl`；域内引用源码拼写：`static Deep deepField;`、
`void work() throws Err, …`、`return new Deep();`、`local1.baseField = new Deep();`——代码行无
`M1$` 池拼写，标记注释按惯例保留物理描述符）与 `results/M2-fold.jarde.java`（`static class Solo`
+ `new Solo().v()`）。

- 整 jar 重编：`javac --release 8` 0 错误；`java -Xverify:all` M1=`hi`/`ok`、M2=`7`——与原 class
  逐字一致（另在 `tests/member_class_static_folding.rs` 七个验收测试内固定重放，含原 jar 作
  classpath 的既定惯例）。
- 分离呈现 diff：M1 家族七个成员类 + `M2$Solo` 逐字节不变（`results/*-after.jarde.java` 与
  `before/` 对照）。M1User 跨类呈现不变（`class M1User extends M1$Base implements M1$Ctrl`）。
- 预算/取消原子性：`output_bytes-1` 与先取消 token 均不发布任何折叠（测试
  `budget_and_cancellation_stop_the_whole_fold_atomically`）。
- 匿名形态不抢道：根家族含合成成员行（匿名 `X$1`/局部类形）时折叠整体拒绝，保持 nn 片平铺惯例
  （`nested_type_source_spelling::synthetic_anonymous_names_stay_verbatim` 继续逐字通过）。

## 4. 三方对照（task 3.2）

固定 JADX（dev 构建 `~/workspace/testzone/jadx/…/bin/jadx`）输出在 `jadx/`。逐路径
`java -Xverify:all` 运行：

| 路径 | 原 class | JADX 重编 | Jarde 折叠重编 |
| --- | --- | --- | --- |
| M1 | `hi`/`ok` | `hi`/`ok` | `hi`/`ok` |
| M2 | `7` | `7` | `7` |
| FV1/FV2/FV3 | `5`/`3`/`6` | `5`/`3`/`6` | `5`/`3`/`6` |

输出 SHA：`results/sha256.txt`。

## 5. corpus 双腿扫描（task 2.2/3.1）

`openspec/evidence` 下 80 个 jar、341 个类，主线基线二进制（`4da1fe84`，临时 worktree 构建，已
清理）与本片二进制逐类 diff：**19 个类变化，全部为折叠家族形态**（新增 `static
class/interface` 嵌套声明 + 域内改写）；其余 322 类逐字一致。变化类重编核查：

- 折叠单元可编：`Other`(VN4)、`p.A`(VN2)、`dt29.SamePackageParentFamily`、
  `MixedShortCircuitField`。
- `p.VN2`：域内孙代引用保留池拼写（登记缺口，实现前同样不可编——旧文本 4 处符号错，新文本
  1 处：`Mid` 引用已改写，仅孙代 `p.VN2$Mid$Leaf` 保留）。
- `SDAbstract`/`SDIndirect`、`H2`(hier-neg)：子类带"未恢复方法体（仅引注）"或负例 jar 缺失
  超类——按 design 决策 3 接受带引注子类折叠（引注语义不隐藏）；这些子类自身单类输出实现前
  同样不可编。

## 6. 登记缺口（后续片）

- **孙代引用**：折叠作用域内对孙代（`Inner$Leaf`）的引用保留池拼写，折叠单元单独不可编（FV5、
  p.VN2 形态）；待孙代折叠或跨单元拼写片。
- **非静态成员类**（this$0 捕获）不做（`recover-member-class-inner-folding` 后续片）。
- **子类 `Signature`**：泛型成员类拒绝折叠（窄通道自管其单变量切片）。
- **带引注子类**：未恢复方法体的子类照常折叠嵌入，引注保留（SDAbstract 形态）。
- **根/子带初始化投影、数组辅助投影**的物理文本复现检查失败即拒绝折叠（保守）。
