# CF11 下一固定候选：顺序循环 oracle 更正 v4（root静态审查）

## 更正范围

本稿保留 v2 原件不变，并更正其中 `TestSequentialLoops2` 的路径推导。v2 的错误有两处：它把单字符 `'a'` 说成可直接返回原串；事实上首次循环会对 ASCII 值 97 执行 `lowercases[97]`，而数组只有一个元素。它也把全非 ASCII 输入描述为第二个循环解引用空 `c`；事实上首次循环耗尽后 `i == -1`，第二循环条件立即失败，`c` 仍为 null，方法返回原串。

## 按源码逐步推导

固定上游源码 `TestSequentialLoops2.java:15-35`：`i = s.length()`；首循环条件为 `while (i-- > 0)`，循环体读取 `s.charAt(i)`；若字符 `<= 127`，先读 `lowercases[c1]`，仅当值不同才分配 `c`、写入并 `break`。第二循环同样以 `while (i-- > 0)` 开始，访问 `c[i]`。静态初始化在 `:12-13`，`lowercases = {'a'}`，所以仅索引 0 合法，且映射值为 `'a'`。

令输入长度为 n。首循环条件先取旧 i 与 0 比较，同时将 i 减一；只有旧值大于 0 才进入循环体，体内使用减一后的索引。因此正常循环体索引依次为 n-1 到 0。若所有字符均大于 127，循环体全部跳过映射分支；最后一次条件检查把 i 从 0 减到 -1 后失败。第二循环入口再把 i 从 -1 减到 -2 并失败，完全不进入循环体；`c` 仍为 null，三元表达式返回原始 `s`。空串也以同样的空 `c` 路径返回原串。

若遇到 NUL（数值 0），查表得到 `'a'`，值不同，代码先分配完整副本、写入当前索引再跳出首循环。此时 `c` 已非 null。若 break 索引大于 0，第二循环从前一索引继续向左处理前缀；NUL 仍映射为 `'a'`，大于 127 的字符跳过。第二循环并不会处理 break 位置左侧以外的字符，也不会遇到未分配的 `c`。

若遇到非 NUL ASCII 字符，代码直接按字符值查表。唯一合法索引是 0；其他值（例如 `'a'` 的 97）抛 `ArrayIndexOutOfBoundsException`。索引 0 对应 NUL，且 `'\0' != 'a'`，故会分配副本并写 `'a'`。因此普通 ASCII 字符不会因为“等于映射值”而直接返回。

## 可用的精确 oracle

以下是从源代码逐步推导的静态 oracle，尚未用编译类或运行时验证：

| 输入 | 预期 | 路径依据 |
|---|---|---|
| 空串 | 原串（空串） | 两个循环均不进体，`c == null` |
| `"\u0080"` | 原串 | 首循环跳过所有非 ASCII；i 降至 -1；第二循环不进体 |
| `"\0"` | `"a"` | 索引 0 查表成功并分配副本 |
| `"\0\0"` | `"aa"` | 首循环改写索引 1 后 break；第二循环改写索引 0 |
| `"\u0080\0"` | `"\u0080a"` | 首循环直接在索引 1 的 NUL 分配并改写后 break；第二循环在索引 0 跳过 U+0080 |
| `"\0\u0080"` | `"a\u0080"` | 首循环跳过索引 1 的 U+0080，再改写索引 0 |
| `"a"` | `ArrayIndexOutOfBoundsException` | 首循环访问 `lowercases[97]` |
| `"a\0"` | `ArrayIndexOutOfBoundsException` | 首循环在索引 1 的 NUL 分配副本并 break；第二循环在索引 0 访问 `lowercases[97]` |
| `"\0a"` | `ArrayIndexOutOfBoundsException` | 首循环先在索引 1 访问 `lowercases[97]`，尚未到 NUL |
| null 引用 | `NullPointerException` | `s.length()` |

若输入含多个 NUL 与非 ASCII 字符，结果应由相同的后缀顺序逐步模拟；不能将这个单元素映射表当作一般 ASCII lowercase 规则。可选控制集合优先使用 `"\0"`、`"\0\0"`、`"\u0080\0"`、`"\0\u0080"` 和全非 ASCII `"\u0080"`，再将上述异常输入单列为失败路径。异常预期也必须在未来实际运行中确认；本报告没有声称已确认。

## 候选排序结论不变

下一固定项仍推荐 `TestNestedLoops2.TestCls.test(List<String>)`，理由是它有清晰的最大字符串长度阈值 oracle，且与已接受的 NestedLoops4 side-exit 实例有不同的循环形状。`TestSequentialLoops2` 仍排后：经更正后，它确有可预测的小型输入，但正常映射行为只适用于 NUL，其他 ASCII 输入会触发退化查表异常；它更适合作为后续固定案例，而不应被描述为完全没有 runtime oracle。只有真实编译类三方回放后才能决定它是否值得进入产品差异处理。

## 来源与精确 pins

- 上游固定测试：[TestSequentialLoops2.java](/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/loops/TestSequentialLoops2.java:12)，SHA-256 `adb34f80f7d55cd3309ed7708f0fdb46768cf32e4004dbd13d4e278530306720`。
- 被更正的私有旧稿：`/private/tmp/jarde-cf11-next-fixed-order-luna-v2.md`，SHA-256 `39ab0a5b2d727d86ff8058f393327209f8a3655e3050a8a13251b52af5eafa31`。
- 本轮原候选报告 v1 仍按 v2 中的说明保留；v2 的 `TestNestedLoops2` 推导和账本/CF11历史状态未在本次改写。
- 静态源码证据位置：`TestSequentialLoops2.java:12-13` 数组定义；`:15-27` 首循环及首次分配；`:29-35` 第二循环及返回。源文件未修改。

本稿仅完成源码控制流推导；未运行 JDK、JADX、CLI、Cargo 或候选编译类，未修改共享仓库。因此以上输入、返回值和异常仍是待真实三方回放验证的候选 oracle，不计为通过。

Root v4 additionally corrects v3 postfix-condition wording: compare the old i, decrement on both true and false checks, then use the decremented i in the body. It also corrects the mixed NUL/U+0080 traversal order; the proposed outputs are unchanged and unexecuted. v3 remains preserved.
