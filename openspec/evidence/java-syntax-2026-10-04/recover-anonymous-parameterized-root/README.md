# recover-anonymous-parameterized-root 实施证据（2026-10-04，coder）

环 3 的完整取证：主锚 `anonymous-super-dispatch` 从 `anonymous_super_return_type_unproved`
拒绝（渲染源集 `javac --release 8` exit 1）转为投影（渲染源集 exit 0、运行输出逐行一致），
`-g:none` 同形对照腿同样转变，五个负例全部保持物理文本并响亮拒绝，corpus 双腿扫描差异恰为
锚与其同形腿两处。实现 tip：`9f0cd47d`（基线 `c7dec2fd`）。

## 目录

| 路径 | 内容 |
| --- | --- |
| `baseline/` | tasks 1.1 重放：`javap-root.txt` / `javap-child.txt`（design Context 逐项复核）、`rendered-root-before.txt`（物理文本）、`report-before.json`（拒绝码）、`source-set-before/`（渲染源集 javac exit 1）、`original-class-run.txt`（`observed=captured-value` / `visibleDuringSuper=true`）、`anchor-input.zip` |
| `negatives/` | tasks 1.3 五类负例：每项的 `before-*.txt` / `after-*.txt`（渲染前后逐字节 cmp 为空）、`before-*.json` / `after-*.json`（拒绝码对照）、`javac-*/`（渲染源集 javac exit 1 实测）、`*-input.zip` |
| `fixed/` | tasks 3.1：`rendered-root-after.txt`（`-g` 腿投影）、`source-set-after/`（javac exit 0 + 运行输出）、`rendered-run-after.txt`、`rendered-nodebug-after.txt`（`-g:none` 腿投影，`arg0`）、`source-set-nodebug/`、`rendered-nodebug-run.txt` |
| `corpus-two-leg-scan/` | 99 渲染/腿（`old/` 基线二进制、`new/` 实现二进制）、`diff.txt`（恰两文件 + SUMMARY）、`anchor-diff-content.txt` |
| `../anonymous-chain-rings-2-3/` 同级的 `two-leg-lvt-counts.md` | 双腿 LVT 计数表（本目录内） |
| `SHA256SUMS.txt` | 全部证据文件摘要 |
| `report.md` | 交付报告：五条决策落点、退化形、参数名来源证明、门禁数字、遗留缺口 |

## 关键数字

- 主锚 `-g` 腿：拒绝码 `anonymous_super_return_type_unproved` → 投影 `return new Base() { … }`；
  渲染源集 `javac --release 8` **exit 1 → exit 0**；`java -Xverify:all` 输出与原 class
  **逐行一致**（`observed=captured-value` / `visibleDuringSuper=true`）。
- `-g:none` 腿：同一转变，参数名来自 AST 命名通道的发明名 `arg0`（LVT 计数 0，对照表
  `two-leg-lvt-counts.md`），javac exit 0、输出逐行一致。
- corpus 双腿扫描：**99 渲染/腿，差异恰 2 处**（锚 + 同形 `-g:none` 腿，同一差异类）；
  环 1 遏制负例 `anonymous-local-decl-interface-hold` 渲染 SHA-256
  `1badfcb5b9dcb9a46bf017e3b285073e8424c8143a239f3a6bac9606efc98ce5` 与冻结基线**逐字节相同**。
- 门禁：全仓测试 297 目标 / 2947 passed / 0 failed（两轮）、fmt 干净、clippy（ci.yml 46–76
  逐字生成，29 项 `-A`、`--all-features`、`-D warnings`）exit 0、`openspec validate --all
  --strict` 271/271、`git diff --check` 干净。
