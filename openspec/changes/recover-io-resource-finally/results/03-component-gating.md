# 分部件门控（四个配置，同一探针集）

命令：`sh results/03-component-gating.sh`（脚本自备份/自恢复 `build.rs`、`init.rs`；行的开关由
`03-set-rows.py` 幂等切换，深度由 `sed` 切换；每个配置重编译 CLI 后跑同一探针集）。基线 = 父提交
`c99e26a5` 的二进制（LK / void-loop 的"identical"是与它逐字节比）。

| 配置 | io（`countLines` 呈现） | mid（`IOMidRead` 呈现） | widening（两实参位 cast） | depth（`threeLayer` 单表达式） | lk | void-loop |
| --- | --- | --- | --- | --- | --- | --- |
| full（证书+宽化行+深度 3） | 1 | 1 | 1/1 | 1 | identical | identical |
| cert-only（仅证书） | 0 | 1 | 0/0 | 0 | identical | identical |
| cert+rows（证书+宽化行） | 0 | 1 | 1/1 | 0 | identical | identical |
| cert+depth（证书+深度） | 0 | 1 | 0/0 | 1 | identical | identical |

（`widening` 列为"第一行实参位/第二行实参位"各自是否呈现；0/0 表示两处都拒。）

## 结论

1. **证书单独翻转同形**：`IOMidRead.countRemaining` 在 `cert-only` 即呈现——行集证书本身承认该
   形状；`countLines` 在 `cert-only` 仍拒，说明它还需要另两部件（构造链的实参位与深度）。
2. **宽化行各自单独生效**：`cert+rows` 里两个实参位都呈现，`cert-only`/`cert+depth` 里都不呈现
   ——两行是它们自己的位点的必要且充分条件（表单元测试另行证明没有别的行/边能到达这两对，
   见 [04-widening-rows.md](04-widening-rows.md)）。
3. **深度单独生效**：`cert+depth` 里 `threeLayer` 呈现，`cert+rows` 里不呈现。
4. **不变量在每个配置成立**：LK 与 CF-16 固定形在四个配置里都与基线逐字节相同——证书既没有夺取
   LK 单行形状，也没有夺取固定 void-loop 形（后者由"完成形只收 `SavedReturn`"保证）。
5. `countLines` 需要三者同时在场：`full` 是唯一呈现它的配置（实测，非推断）。
