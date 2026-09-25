## 1. 冻结三方现状

- [x] 1.1 重放 `method-type-variable-shadowing/replay.py`，验证 Java 8 `-g`/`-g:none` 原 class 两行运行结果、`javap` 类/方法 Signature、Jarde 两例强类型调用方重编失败，以及 JADX 无界通过/异界失败的独立诊断和 SHA 清单。

## 2. 修正同一 reader 证明链

- [x] 2.1 让方法变量遮蔽已证类变量并按方法第一界擦除，保持各作用域内部重名和循环界拒绝；以 reader 定向测试核对无界、`Number`/`CharSequence` 异界、方法同层重复、未绑定及预算/取消。
- [x] 2.2 核对方法参数、返回与 `Exceptions` 的每个已解析位置都按新优先级对物理 descriptor/属性验证；用改界后擦除错误的 verifier-valid 签名控制和原有类 T/方法 U 回归确认拒绝及不回归。

## 3. 静态直接返回源码闭环

- [x] 3.1 复用现有类头发布和静态直接参数返回候选，把 `<T> T echo(T)` 与 `<T extends CharSequence> T echo(T)` 原子投影；用两份 Jarde 完整 class-source 的 Java 8 重编和反射类/方法泛型签名核验，不放宽非静态或复杂正文。
- [x] 3.2 将未改的 `StrongCaller.java` 与 Jarde 两份完整类源码一同 Java 8 编译并运行，`-g`/`-g:none` 均与原 class 输出 `plain`、`bounded`；确认 JADX 仅无界样例通过同一强类型调用方验收。

## 4. Root 独立验收

- [x] 4.1 重跑定向 reader/Jarde 测试、证据选择与预算/取消、格式和 `openspec validate --strict`；检查共享树既有门禁不混入本改动，清理私有 Cargo target，记录 `verification-root.md` 后只勾选实测完成项。
