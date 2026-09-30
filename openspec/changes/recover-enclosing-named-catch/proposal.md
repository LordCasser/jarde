## Why

[CF-17 巡查](../../evidence/java-syntax-2026-09-30/cf17-twrcatch-patrol/README.md)确认：`try (r) { … } catch (E e) { … }` 的 javac 具名行覆盖整个 TWR 降低（init 到 cleanup），被 `guard.rs::enclosing_clauses` 的 `!(row.start_bci <= shape_start && row.end_bci >= handler_end)` 排除子句显式拒绝（`jre_guard_unexplained_row`）——规则为防"TWR 伪装用户 catch 丢清理语义"而保守。scratch 实验证实：放宽该排除后 guard 拒绝消失、TWR claim 成立，唯呈现层不发射 catch 子句致 handler 块未覆盖。本片闭合该高频形态（T3.voidNamed/voidNamedRecover、T1.twrVoidNamed、C4.twrNamed；与 17a 叠加形状随之闭合）。

## What Changes

- `enclosing_clauses` 接受"完整覆盖 claim 跨度的具名行"为包围子句（守卫：行覆盖 `[claim 起点之前的语句边界, handler_end]`、具名类型、handler 块在 claim 外、claim 自身行集合不变）。
- TWR 的 Plan/Region/Builder 发射包围 catch 子句：`try (…) { … } catch (E e) { handler 体 }`——handler 块作为子句体走既有 region 呈现，进入覆盖账本；catch 参数为 handler 的绑定 store。
- 17a 落地后实施；多 catch 子句（`catch (E1 | E2)`）不在本片（负例钉死，随后按需扩）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证 TWR + 包围具名 catch 恢复为 `try (…) { … } catch (E e) { … }`，handler 体与资源语义都完整呈现。

## Impact

仅 `crates/jarde-java` 私有 Guard（enclosing_clauses 与 TWR Plan 携带包围子句）、Region/Builder（catch 子句发射与覆盖）及测试；TWR 无 catch、catch 包普通 try（Catches 既有路径）行为不变。不新增公开 IR/CLI/依赖。
