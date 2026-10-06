- [x] 1.1 重跑巡查 fixture（OB.java javac8）记录基线渲染与诊断
      → 实测：`ob.jar`（巡查字节）与 `OB.java` 真 javac 8 重编后同形；基线拒绝 = BCI 6（`java.lang.Enum` 位）+ BCI 12（`java.util.Collection` 位），`main` 另有 BCI 79/85/87 的嵌套数组/copy 家族 7 行。
- [x] 1.2 Q-i：Enum 扩宽 checkcast 在渲染管线的现行投影点与三张接口表的表键差异（类位 vs 接口位是否同门）；Q-ii：无枚举调用点字节级零漂移的验证面
      → Q-i 实测（判别实验）：`java.lang.Enum` 位在 HEAD **已由快照单边证明覆盖**（容器含 `OB$Flag.class` 时该拒绝消失；`--policy single-class` 时逐字回归），**不需要也无法封闭枚举用户枚举类名**；真实剩余拒绝是 `EnumSet → Collection`（接口位，同一 widen 通道）。Q-ii：`Objects.*` 三成员文本与巡查记录逐字节一致（冻结 fixture `EN` 中断言）。
- [x] 2.1 平台扩宽表补 java.lang.Enum 行（含 EnumSet.of 变长形），按姊妹表协议（javadoc 行/hex/三方一致）
      → **偏差（实测为准）**：落表的是**枚举族的 `java.util.EnumSet` 自身的 javadoc 行**（extends `AbstractSet`；implemented-interface 达 `Set`/`Collection`/`Iterable`），因为 `java.lang.Enum` 位已由快照通道作答（上面 Q-i）；变长形（`of(E,E,E)`）与 2 参形在冻结 fixture 中分别钉住。
- [x] 3.1 全门禁 + fixture 双协议（人口计数 + corpus 指纹）
      → 见本片 verification“门禁”节：人口计数按约定改为 `(687, 2932, 282, 1827, 8)`（+22 类/94 body），语料指纹再生。
- [x] 3.2 四表合并派发协调（与 charsequence/comparable/serializable 同一实现片）
      → 一次实现（同一函数四张表 + 一个数组位谓词）、一个测试入口（`tests/recover_platform_implementer_argument_widening.rs`）、分逻辑提交。
- 注：本 change 的 spec Scenario 1 文字“无 `(java.lang.Enum)` cast 呈现”与**既定机制**（`cast_argument` 呈现）不同——既有快照通道与本片表命中一律经 `cast_argument` 呈现（`(java.lang.Enum) OB$Flag.A` 形）。本片按机制落地并逐字钉住实测文本；若 spec 要改为“无 cast”，那是呈现层的另一个决定（会波及全部既有通道的呈现约定）。
