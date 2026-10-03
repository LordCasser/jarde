# 消隐前置不变量的 javac 实证（2026-10-04，root）

隐藏 javac 合成成员的前提是源码能让 javac 重新生成同一合成物；擦除桥的可重建性来自类头类型参数投影，不来自平台事实。

## 裸 implements + 隐藏桥 → 硬编译错误
```
public class RawI implements java.lang.Comparable {
    public int compareTo(RawI o) { return 0; }
}
```
```console
$ javac --release 8 -Xlint:-options RawI.java
RawI.java:1: 错误: RawI不是抽象的, 并且未覆盖Comparable中的抽象方法compareTo(Object)
public class RawI implements java.lang.Comparable {
       ^
1 个错误
```

## 参数化 implements + 隐藏桥 → javac 自行重建桥
```
public class ParamI implements java.lang.Comparable<ParamI> {
    public int compareTo(ParamI o) { return 0; }
}
```
```console
$ javac --release 8 -Xlint:-options ParamI.java   # exit=0
$ javap -v -p ParamI.class | grep -c ACC_BRIDGE
1
```

## 判据范围的 2x2 对照（2026-10-04，root 独立验证）

验证 `recover-bridge-admission-gates` 实现所用的前置条件范围（`InvokeInterface` + 参数 cast 形）是否与真实失败形精确重合：

| 源码形 | \`javac --release 8\` | ACC_BRIDGE | 结论 |
| --- | --- | --- | --- |
| \`Cov\`：非泛型接口 + 协变返回 | exit 0 | 1 | 裸头下 javac 仍重建桥 → **不需前置** |
| \`CovGen\`：泛型接口裸头 + 协变返回 | exit 0 | 1 | 同上 → **不需前置** |
| \`SupNarrow\`：父类 + 参数收窄 | exit 0 | 0 | 裸头使收窄覆写退化为普通重载，仍可编 → **不需前置** |
| \`IfaceNarrow\`：泛型接口裸头 + 参数收窄 | **错误：未覆盖 put(Object)** | 0 | **仅此形需前置** |

即前置条件的正确范围是「接口边 ∧ 参数 cast 形」——与实现所用的 `*use_kind == ReferenceUse::InvokeInterface && parameter_cast_form` 精确重合；协变返回形与父类边均不应被该前置拦住（否则会把当前已可编的两类输出退化为不可编，构成反向回归）。
