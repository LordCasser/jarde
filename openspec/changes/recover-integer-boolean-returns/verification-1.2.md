# 任务 1.2：Java 8 整数返回 boolean fixture

`tests/fixtures/p3-integer-boolean-returns/IntegerBooleanReturns.java` 是唯一 source-only 目标。
它用 Java 8 编译为全 `int` 返回，再由脚本只改 `direct(I)I`、`once(IntegerBooleanReturns,I)I`、
`post()I`、`pre()I`、`sync(IntegerBooleanReturns,I)I`、`syncOn(Object,I)I` 的返回描述符为 `Z`。
`post`/`pre` 分别是已经支持的字段后自增/前自增形状。`next` 只作为被调用的 `int` 方法，增加
`calls` 计数，以 runner 观测调用次数。class SHA-256 为
`5b993e7c5b28588ded97dc53b6982608895745fdd625a77e0feb69ee6783eade`；补丁前后全部方法 Code
SHA-256 均为 `347f163f2497b8b8cb0a3cd7f246f0ee9db9179a4c7660d68df506301bdff854`。
逐项 descriptor 与 class hash 见 `tests/fixtures/p3-integer-boolean-returns/v8/IntegerBooleanReturns.patch.json`。

直接返回、单次调用、字段完整更新、同步和 null 异常运行结果冻结在
`tests/fixtures/p3-integer-boolean-returns/v8/expected.txt`。`javac --release 8 -g:none` 成功；对补丁
class 运行 `java -Xverify:all` 成功。覆盖 0、1、2、3、-1、-2、`Integer.MIN_VALUE`、
`Integer.MAX_VALUE`；调用计数从 1 到 8；post/pre 各自输出 boolean 返回和更新后的完整字段整数值；
同步方法对全部数值保留最低位，`syncOn(null, 1)` 得 `NullPointerException`。

共享 switch 边界由已有真实操作数栈 join 的 major-49 class 派生，只改
`runByte(I)B` 为 `runByte(I)Z`。两臂仍在 operand stack 合流并共用 BCI 32 的 `ireturn`，本例不涉及
条件 stack phi。class SHA-256 为
`3b727e5b7bb8b630eca95ada5f0ff6516063e58fa94718ba36cf1c649ca107e2`，补丁前后 Code SHA-256 为
`0ce0e123f556ebfeb08ada927a60db37218f7ec5c7ec407df2d3c51f61c26a12`。`-Xverify:all` 结果为
`-1:true`、`0:true`、`1:false`、`2:true`（两 switch 臂值分别为 130 与 -129）；hash、补丁和结果在
`v8/actual-stack-join/`。

从仓库根目录执行以下命令可在独立目录重建 staged 输出；脚本断言 descriptor 补丁前后 Code hash
完全相同，runner `.class` 只生成在指定输出目录。与永久文件逐字节复比时，两个目标 class、两个
patch JSON 和两个 expected 输出均相同：

```sh
python3 tests/fixtures/p3-integer-boolean-returns/regenerate.py /tmp/jarde-intbool-gen
```

JADX 1.5.6 原样完整输出保存在 `v8/IntegerBooleanReturns.jadx.java.txt`。`jadx -d ...` 成功生成，
但其原样源码在 `javac --release 8` 阶段失败：对 `post` 和 `pre` 输出了非法的 `?? r1` 声明（6 个编译
错误）。未修补该输出。Jarde 测试使用独立 `CARGO_TARGET_DIR=/tmp/jarde-intbool-fixture-target`：新 fixture
的 direct、once、post、pre、sync、syncOn 均为 Java/Structured，方法来源覆盖真实返回 BCI；完整类无引用，
重编译后以 `-Xverify:all` 执行并逐字节匹配原 runner 输出；共享 switch 测试覆盖 BCI 1、20、26、32，并
单独确认 BCI 32 的直接来源。

**修复前阶段基线不确定。** 本 fixture 建立时共享工作区的生产代码已存在并行修改，无法证明本次 Jarde
完整类结果来自修改前版本；因此不把当前绿色结果声称为修前 RED。曾尝试的复合字段更新样例不在已支持的
字段自增形状内，已替换为本 change 指定的前/后自增 fixture，未将该试验作为正例证据。
