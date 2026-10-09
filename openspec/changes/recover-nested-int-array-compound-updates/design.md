## Context

动机见 proposal.md。P02 helper 的物理 descriptor 为 `([[ILjava/lang/Integer;)V`，Code 为 aload0@0、常量@1、aaload@2、常量@3、dup2@4、iaload@5、aload1@6、intValue@7、iadd@10、iastore@11、return@12。原数组 [[I 经现有 array_of_value 的 ArrayElementLoad 降一 rank，可证明原始行值 int[]；aaload frame 为 Unknown Ref，dup2 解码为 Other，复制值不能独立追回行来源。

root fresh JADX 1.5.6 两输入 × default/none rename profiles 共四腿完整提取、真实 JDK8/23 空CP/SP重编、仅新classes运行均匹配原程序 `6\n`；JADX 生成局部行数组后普通赋值。版本、全生成源、命令、原双流与 78 项独立核验保存在前片 results/nested-array-jadx-baseline-v2。v1误把rename profile传给decompilation-mode导致extract1，保留为失败记录。用户指定源码checkout为 v1.5.6-26-g2fb1b163，与执行安装版本明确区分。

## Goals / Non-Goals

**Goals:**

证明既有 int-add compound 的真实行对象身份，恢复嵌套下标并保留原语义、来源和原子停止。覆盖原P02和 bounded 二维/三维、旧一维控制。

**Non-Goals:**

不扩展操作符、其它primitive、全局dup2推导或array_of_value到任意调用/合并来源；不改lambda、SAM、capture和初始化。不宣称JADX测试文件清单已经覆盖多维compound，也不把本片当DT26全部完成。

## Decisions

### 在复制身份闭合处使用原始行类型

现有 prove_array_update 在读取 store operands 后对 array_store 副本要求 int[]，随后又对 dup2 原始 array 要求同类型，并证明四个各异输出、store/read准确对应、唯一用途与时序。仅删除冗余副本类型gate，保留原始值类型证明和全部身份约束；这恢复证明链，并非对 Unknown 引用无条件放行。替代方案在frame全局传播dup2的类型或新增数组类型服务影响所有消费者且本片不需要，故不采用。现有IndexAssign/IndexExpr与来源遍历足够，无新增实体。

### 不照搬重复数组访问的文本

JADX InsnGen 的 APUT 输出 array[index]=value，实测通过临时行变量保留身份；其 SimplifyVisitor convertFieldArith 针对field，不是多维array compound规则。借鉴准确左值/旧值身份与求值保存原则，jarde复用已证明的 `+=` AST表达式，使行、index、RHS各求值一次且 iaload 的null/越界先于RHS。无需复制JADX Java实现，也无外部库/新license依赖；标准现有Rust表示与JVM SSA事实已提供所需能力。

### 完整类与真实副作用验收

P02 sum/capture、main、constructor和adapter须保持，只升级helper拒绝断言；历史baseline/fixed文本不改。新完整控制类覆盖plain2/plain3/scalar、带调用row/index/RHS、null/OOB、RHS换外层行后仍写原行、不同读写行。Runner为同一固定外部测试harness，candidate编译全部生成target源与该Runner，不能借原target类/helper；目标每个原成员必须存在。原source实际双JDK编译/javap/执行先冻结，JADX与candidate按全部源同样执行并比较原exit/raw stdout/stderr。不同读写普通赋值若未证明允许沿原拒绝契约降级，但绝不能错误fold成compound；将该控制独立记录，不把其失败删除以制造全类成功。

### 保持现有停止和所有权契约

array_of_value保留既有MAX_VALUE_DEPTH bounded追溯；不声称它原本逐节点Budget计费。后续collect_expression_bcis/用途区间检查透传同次Budget/Stop，计划完整闭合后移交，失败不能消费dup2/read/dependencies。预算、取消和错误复制/消费的生产控制及source-map验收沿当前通路；若发现无关计费债务独立记录。

## Risks / Trade-offs

- 副本类型gate删除可能扩大接纳 → 必须保持原始int[]类型、准确四副本与iaload/iadd/iastore证明，加入错误row/index与unknown来源负例。
- 文本看似正确却重复求值或移动异常 → Runner记录trace，outer null/OOB为1，null row/innerOOB为12，成功为123，换行控制保存旧行引用并检查两行值。
- 新完整源中既有其它恢复边界阻塞 → 保留所有失败和具体成员定位，控制与产品边界分开，不通过剥离成员或放宽断言验收。
- 旧CI被当新产品验证 → BigDecimal修正提交8cd8c4f5fbfdb51d56c3725783e2d0666f8903ac的CI37991578328独立按immutable Gitblob与旧冻结CLI核验；当前WIP的不同hash仅记录、不能借它验收。root原五源暂冻限制到旧CI校验后可由不可变commit证明代替，避免无谓串行等待；新产品仍单独冻结CLI、提交和验收确切新CI。root串行Cargo，20GiB停止线，清本仓target。
