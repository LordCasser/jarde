# 生产码中硬编码方法名 `"make"`（2026-10-04，root 巡查）——已被相邻臂掩盖，非用户可见缺口

**结论：`crates/jarde-java/src/init.rs:833` 在生产判据中硬编码了方法名 `method_facts.name() == "make"`，且全仓无任何文档记载该窄形。但 root 未能构造出使其产生用户可见差异的形状——同一判据的相邻臂（enclosing-`this` 限定符臂，`init.rs:888-928`）在 `make` 之外的名字上产出逐字相同的呈现。故这是**代码卫生/可维护性缺陷（未被文档记载的硬编码标识符）**，不是能力缺口。与已修复的 `op` 字段名硬编码属**同一失败模式类**，但严重性显著更低：`op` 那起是用户可见的（非 `op` 字段名直接不投影），本起被掩盖。**

固定转录见 [fixture](fixture/)（`Mk.java` 四同形方法）与 [results](results/)。

## 一、缺陷事实（读码核实）

`init.rs:829-883`（`fn verify_member` 的第一臂）：

```rust
if let (Some([receiver, call]), Some(method_facts)) = (block.get(index + 2..index + 4), method) {
    let root_descriptor = [b"()L".as_slice(), target.owner.as_bytes(), b";"].concat();
    let exact_root_return = method_facts.name() == "make"          // ← 833：硬编码方法名
        && method_facts.descriptor().as_bytes() == root_descriptor
        && method_facts.access_flags().is_some_and(|flags| flags & 0x0008 == 0)   // 非 static
        && method_facts.declaring_class().is_some_and(|class| class.name() == target.outer);
    let exact_code = code.instructions.len() == 5
        && code.stopped_at.is_none()
        && code.instructions.iter().map(|i| i.opcode).eq([0xbb, 0x59, 0x2a, 0xb7, 0xb0])  // new dup aload_0 invokespecial areturn
        …
```

即该臂只接受**恰好名为 `make`**、描述符为 `()Lowner;`、非 static、声明于 outer、方法体恰为 5 条指令 `[new, dup, aload_0, invokespecial, areturn]` 的形。

**归属核实**：

- `init.rs:833` 在**生产区**（`init.rs` 的测试模块起点为 **1832**，root 用 `awk 'NR<1832'` 与 `NR>=1832` 分段计数：生产区 `name() == "make"` 命中 **1**、测试区 `"make"` 命中 **7**）。
- 全仓 `"make"` 字面量的其余命中（`init.rs:2231/2258/2266/2275/2284/2296/2309`、`build.rs:27252/27260`）**全在测试区**（`build.rs` 测试模块起点 26830，27252 > 26830）。故生产码只有 `init.rs:833` 这一处。
- **无任何文档记载**：`grep -rln '"make"' openspec/changes/*/design.md openspec/changes/*/proposal.md` → 0 命中；该臂所属的 `inner-generic-instance-constructor` 巡查证据亦未把"名字恰为 `make`"记为窄首片限制。故它既不是"文档记载的可接受窄首片"，也不是"语言/JDK 强制名"（Java 对工厂方法名无任何约束），属**未登记的硬编码**。
- **root 自查纠错**：本节初稿把 `init.rs` 的测试模块边界误写为 **11237**（那是 `src/facade.rs` 的边界，root 串了文件），并据错误边界把测试区命中数写成 3。正确边界是 1832、测试区命中 7 处。**结论不变**（833 < 1832，仍属生产区），但边界与计数已按实测更正——这正是 handoff「空查询不能证明不存在」与「脚手架先自检」纪律的反面教材：**把 A 文件的结构常量套到 B 文件上，会同时污染计数与归属判定**。

## 二、为什么它当前不产生用户可见差异（root 实测）

root 构造 [fixture/Mk.java](fixture/Mk.java)：同一个类内四个**字节码完全同形**的方法，只有名字不同：

```java
public class Mk {
    class In { In(){} }
    In make(){ return new In(); }      // 恰为硬编码名
    In create(){ return new In(); }    // 同形、仅名字不同
    In newInner(){ return new In(); }
    In get(){ return new In(); }       // 常见 getter 名
}
```

四方法的指令序列**逐字相同**（root javap 核实：各 5 条 `new dup aload_0 invokespecial areturn`，`sort -u` 后**只有 1 种序列**）：

| 方法 | 指令数 | opcode 序列 |
| --- | --- | --- |
| `make` | 5 | `new dup aload_0 invokespecial areturn` |
| `create` | 5 | 同上 |
| `newInner` | 5 | 同上 |
| `get` | 5 | 同上 |

合并后二进制家族级渲染（`--class Mk`）：**四个方法全部呈现为 `return new In();`，源码区 quotes=0、not-recovered=0**（[results/Mk-rendered.txt](results/Mk-rendered.txt)）。即名字不同的三个方法由**相邻的第二臂**（`init.rs:888-928`，enclosing-`this` 限定符臂：`block.get(index+2)` 为 `Load{slot:0}` + 单一写 = `physical_outer` + `single_use_at` + `names_outer`）接管，产出与第一臂**逐字相同**的呈现（两臂的 `implicit_this` 均为 `true`）。

**故硬编码被掩盖**：第一臂对该形是**冗余**的（第二臂覆盖同形的其余名字），删除或保留 `name() == "make"` 都不改变可观察呈现。root **未能**构造出"第一臂接受、第二臂拒绝"的形状，因此**不宣称存在能力缺口**。

## 三、严重性判定与登记

- **不是能力缺口**（与 `op` 字段名硬编码不同）：`op` 那起导致非 `op` 字段名的枚举 String 实参投影**直接不工作**（用户可见、已修，见 [enum-string-field-name-hardcode](../enum-string-field-name-hardcode/README.md)）；本起被相邻臂掩盖，**无用户可见影响**。
- **是代码卫生缺陷**：生产判据里有一个**未文档化的魔法字符串**，它使第一臂的实际接受面比其代码结构看起来更窄，且任何将来对第一臂的修改都可能因这个名字而产生**不可预期的窄化**（例如若将来第二臂的前置条件收紧，第一臂就会成为唯一路径，届时 `make` 之外的名字会静默失去覆盖——而 CI 无法发现，因为**所有正例测试都用 `make` 这个名字**：`init.rs:2231/2258/2266` 与 `build.rs:27260`）。
- **与 `op` 缺陷同根**：都是"验收锚恰好使用了硬编码的那个标识符"，故硬编码对 CI 不可见。这正是 handoff「验收锚不得是唯一正例——标识符泛化须有对照探针」纪律要防的情形；`op` 那起修复时已补了非 `op` 对照 fixture，本处**尚无对照**（因为当前无用户可见差异可对照）。`init.rs` 测试区共 **7** 处使用 `"make"`（2231/2258/2266/2275/2284/2296/2309）、`build.rs` 测试区 2 处（27252/27260），**无一处使用其它方法名**——故若将来第一臂成为唯一路径，CI 仍不会发现。

**处置**：**登记为独立债务，不立即立项**。理由：无用户可见影响，且修复它需要先回答一个设计问题——**第一臂是否应当存在**（若第二臂已完整覆盖其形，正确处置可能是**删除第一臂**而非把 `"make"` 泛化；泛化一个冗余臂只会增加判据面）。该问题须由一次独立取证回答：证明或证伪"第一臂接受集 ⊆ 第二臂接受集"。

**建议的取证方向（root 未做，不外推）**：
1. 穷举第一臂的接受条件（5 条指令恰等序列 + `()Lowner;` + 非 static + 声明于 outer + `receiver_value == physical_outer` + `single_use_at(call.bci())`），与第二臂的接受条件（`Load{slot:0}` 在 index+2 + 单一写 == `physical_outer` + `single_use_at(at)` + `names_outer`）逐项比对，找出**第一臂成立而第二臂不成立**的差集（候选差异点：`single_use_at` 的锚是 `call.bci()` 还是 `at`；`qualifier_writes.len() == 1` 的约束；`names_outer` 对 `physical_outer` 的类型要求）。
2. 若差集为空 → **删除第一臂**（简化判据面，消除硬编码），并为第二臂补一个名字不是 `make` 的对照正例。
3. 若差集非空 → 把 `"make"` 换成**从字节码证明的事实**（该臂真正需要的性质是什么？若只是"方法体恰为那 5 条指令"，则名字根本不该进判据），并冻结差集形为正例、`make` 之外的名字为对照。
4. 两种处置都须补**名字泛化对照**，否则同类硬编码将来仍不可见。

## 四、root 自查纠错（诚实登记：本起取证踩了三次脚手架 bug + 一次未成立的推断；另有第四节外的边界误用已在第一节更正）

1. **假零结果**：root 首次渲染 `Mk` 时用相对路径 `target/debug/jarde-cli` 且已 `cd /tmp/mk`，得到 exit 127，输出文件内容是错误信息 `target/debug/jarde-cli: No such file or directory`；随后对该文件计 `@bytecode` 得 **0**，一度被读成"四方法全部恢复"。实际上那个 0 是**错误文件的 0**，毫无意义。改用绝对路径重跑后才得到真实渲染。——这是本会话第 **5** 次同类脚手架事故（前四次：普查静默跳过 65 类、jar 丢包前缀致渲染全空、`cp` 原始源污染渲染产物、文件名与 public 类名不符），已固化进 handoff：**凡"引注数 = 0"的结论，必须先确认输出文件不是错误信息**（检查 `not a compilable project` 等 jarde 自述头是否在文件中）。
2. **awk 正则不支持 `\s`**：统计四方法指令序列时用 `/^\s+[0-9]+:/`，awk 不识别 `\s`，导致四个方法都报 `instrs=0`，险些被读成"四方法都无指令"。改用 `[[:space:]]` 后得到正确的 5/5/5/5。——与 handoff「验证脚手架必须先自检」同源：**空结果不能自证扫描器正确，须先用已知正例自检**。
3. **一次未成立的推断**：root 一度按"`make` 是 POSITIVE fixture 的方法名 → 该硬编码必然用户可见"推断缺陷严重性与 `op` 同级。实测证伪（第二臂掩盖），故本 README 的结论按实测下调为"代码卫生缺陷"。**教训与 `op` 那起相同**：硬编码标识符的严重性取决于**是否存在其它路径掩盖它**，必须端到端实测，不能只看判据文本。

原 class 为行为基准：`Mk` 无 `main`（root 的探针只为比对四方法的呈现，未做行为验证——因为四方法呈现逐字相同且 quotes=0，无行为差异可验）。
