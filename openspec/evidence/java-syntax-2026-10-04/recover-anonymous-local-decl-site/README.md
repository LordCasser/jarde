# recover-anonymous-local-decl-site 实现与验收证据（coder，2026-10-04）

worktree 基线 `537ff21b`。范围：tasks 1.1–3.2（3.3 留 root）。主锚 `tests/fixtures/proved-java-structure/anonymous-super-args/`（`-g:none` 冻结、无 `LocalVariableTable`）。

## 目录

| 路径 | 内容 |
| --- | --- |
| `baseline/` | 放宽前基线：原 class 事件日志（`java -Xverify:all`，`arg:capture|arg:super-label|arg:super-value|base:explicit:17` / `explicit:17:captured`）、verbatim 渲染（`src/`，`AnonymousSuperArgs$1 local2 = new AnonymousSuperArgs$1(…)` 在 main 中部、后随两条 `println`）、两类源集口径的 javac 日志（root+Base 退出 1 `找不到符号`；含 `$1.java` 的三文件集退出 1 灵活构造器预览错误，与 2026-09-27 既有登记一致） |
| `decl-site-fixed/` | 实现后主锚：渲染（`Base local2 = new Base(…) { … }`、捕获读取重拼 `local1`、后随语句原样）、`javac.exit`（**0**）、`jarde-run.log` 与 `behavior.diff.result`（事件日志与原 class 逐行一致） |
| `containment/` | 判据 5 遏制负例：`old-/new-LocalDeclInterfaceHold.java` 逐字节相同（SHA-256 `1badfcb5…` 两腿一致）、`new-state.txt`（`refused` + `anonymous_interface_site_shape_unsupported`）、`original-run.log` |
| `negatives/` | 全部负例：源级五项的 `old-/new-*.java` 逐字节对照（cmp 为空）+ 各项原 class 运行日志；`patch-derivations.py` 与两个补丁派生（`non-unique-alloc`、`self-alloc`）的渲染、退出码与 routing 记录 |
| `two-leg-debuginfo/` | `-g` 对照腿：`old-/new-AnonymousSuperArgs.java`（物理 → `Base instance = new Base(…) { … }`，左端类型来源两腿一致为 `new` owner）、`javac.log`（退出 0）、`jarde-run.log`（与原 class 逐行一致）、`original-run.log` |
| `corpus-two-leg-scan/` | 双腿扫描：`scan.py`（两腿同一 fixture 集，83 渲染/腿）、`old/`、`new/`、`diff.txt`（差异恰 2 处：锚与同形 `-g` 腿）、SUMMARY 差异仅为新负例的请求退出码（0→4，渲染文本逐字节相同） |
| `report.md` | 实现与门禁数字报告 |

## 关键实测链

1. `javap -l -p`：锚 `-g:none` 腿 LVT 计数 **0**；`-g` 腿 **5**（README 双记录）。
2. `javap -c`：`instance.render()` 编译为 `invokevirtual Base.render:()Ljava/lang/String;` —— owner 是 `Base` 而非匿名类，故判据 3 第三项由既有 owner 普查在字节码层闭合。
3. 主锚：`javac --release 8` **1 → 0**；`java -Xverify:all` 事件日志逐行一致。
4. 遏制负例呈现逐字节相同（SHA-256 相同），接口匿名形 corpus 零差异。
