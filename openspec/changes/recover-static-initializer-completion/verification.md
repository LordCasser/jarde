# 验证记录

2026-09-23，主代理已完成本项功能与边界验收；严格 clippy 的既存区域 lint 单列，不宣称全仓门禁全部通过。

## 已完成项

- `crates/jarde-java/src/emit.rs` 以同一个 `body` 入口驱动 commit/replay。已知 `StaticInitializer` 的最外层末尾 `Return(None)` 被写成真实闭合 `}\n`，return 的 origin 保留在该闭合文本；普通方法、构造器、无声明、嵌套 return 和空块语句计数均有定向测试。
- `crates/jarde-java/tests/p3_patterns.rs` 的真实 `static_initializer_class()` 回归确认 `Test.g = 7;` 后无 `return;`，BCI 5 直接映射到 `}\n`，origin member 是同 class 的 `<clinit>()V`。默认 `RecoveryRequest::new` 的文本相同且 source map 为空；source-map phase 少一个 `IrItems` 时 artifact 保持不变，未支付的尾来源不出现在映射中。
- 自写 Java 8 样例、临时 class patch、原 class/jarde/JADX 的 javac 和 `java -Xverify:all` 结果保存在 `openspec/evidence/java-syntax-2026-09-22/static-initializers/`。直线、条件、循环、try/catch、helper 和普通 void 的 6 类 7 次初始化结果由 `root-after/README.md` 记录；空块、合法提前返回、外部 helper 初始化失败及第二次访问见 `README.md` 与 `boundaries/summary.json`。

## 命令结果

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| emitter unit | 20 passed | `/tmp/jarde-initializer-root-unit.log` |
| static pattern/source-map/预算及相邻 patterns | 46 passed | `/tmp/jarde-root-patterns-final.log` |
| Java package | 176 passed（98+32+46） | `/tmp/jarde-root-java-final-invocation-static.log` |
| class-source CLI | 16 passed | `/tmp/jarde-root-initializer-cli-final.log` |
| library class-source | 16 passed | root 独立验收日志 |
| declarations | 5 passed | `/tmp/jarde-root-initializer-declarations.log` |
| self-written Java 8 whole-class comparison | 6 类 7 次通过 | `static-initializers/root-after/README.md` |
| D1 evidence / D0 instrumented | 8 / 5 passed | `static-initializers/root-regressions/` |
| 全仓 fmt / OpenSpec strict | 通过 / 34 passed | `static-initializers/root-regressions/` |

本项未新增永久class。第二阶段冻结输入的census为 `(88,480,74,227,8)`，fingerprint205项（8新增、0修改、0删除），census通过、fingerprint5通过/1忽略；记录在审计corpus目录，不把后续尚未冻结的throw夹具算成本项输入。

严格 clippy 在既存 `region.rs::try_level` 的 `type_complexity` 处停止；此次失败不能证明其后所有 target 均无 lint。最新日志已保存在 `static-initializers/root-regressions/jarde-root-clippy-next-final.log`。任务3.2要求的执行与债务区分已完成，区域lint保留为独立仓库债务。合法提前返回、final字段名字绑定与接口初始化均不在本项恢复范围，不用删除生成正文来冒充通过。
