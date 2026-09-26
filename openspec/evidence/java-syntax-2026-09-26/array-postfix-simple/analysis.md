`PostfixProbe.java` 的 `readThenIncrement(int[], int)` 是 OpenSpec 2c.16 要求的精确形状：`return a[i]++;`。用 `javac --release 8 -g:none` 编译的冻结 class SHA-256 为 `6b28b8fd3aac2c21b3574a6ac1aeebeaf9379682101da410f7d7da6601e7b37d`。`javap.txt` 中目标方法依次为 BCI 0 `aload_0`、1 `iload_1`、2 `dup2`、3 `iaload`、4 `dup_x2`、5 `iconst_1`、6 `iadd`、7 `iastore`、8 `ireturn`。

| 来源 | 恢复出的目标语句 |
| --- | --- |
| 手写源码 | `return a[i]++;` |
| Jarde 当前 CLI | `return arg0[arg1]++;` |
| 本地 JADX 1.5.6 | `int i2 = iArr[i]; iArr[i] = i2 + 1; return i2;` |

`jarde-method.java` 是 `jarde-cli recover --evidence all` 的原始**单方法**文本，报告为 `quality=structured`、`execution=complete`，但 `syntax_status=unchecked`；它不是可独立编译的整类。报告的表达式来源以 BCI 7 为主、BCI 0–6 为派生，返回语句另映射 BCI 8。`jarde-wrapper.java` 仅将这个未改写的返回表达式放进与原源码相同的最小类和 runner，以隔离验证该方法。原 class、该 wrapper 和 JADX 整类都用 Java 8 编译并经 `java -Xverify:all` 执行，输出均为 `old=41,new=42`。JADX 的三步写法语义正确；Jarde 在此形状的句法更简洁。

当前实现由 `build.rs::PostfixUpdates::prove` / `prove_postfix_array` 使用 SSA 身份约束数组、下标、旧值返回及新值写回，并非只匹配 opcode 序列。更广的有副作用 `array()[index()]++`、null/越界/溢出与八个 verifier-valid 反例已在 `recover-postfix-lvalue-values` 变化中验收。因而 2c.16 是计划状态未同步，而不是新的实现缺口；这份最小对照补上其字面指定的参数数组形状。
