# 数组协变存入巡查（2026-10-06 root）——安全拒形：窄化局部类型使异型存入不可编译

## 探针

[fixture/AS.java](fixture/AS.java)（`--release 8`）：`Object[] a = new String[2]; a[0]=Integer.valueOf(1)` 与 `Number[] n = new Integer[2]; n[0]=Double.valueOf(2.5)`（协变数组 + ArrayStoreException 运行时检查）+ 同型对照。

## 发现

- **行为面安全**：运行时 ASE1/ASE2 精确（原类实测）；
- **呈现窄化**：局部被推断为分配组件型 `String[]`/`Integer[]`（无 LVT 时字节码只有 `anewarray` 组件与 `aastore`，源声明的 `Object[]`/`Number[]` 不在事实里），异型存入渲染为 `String[0] = Integer…` / `Integer[0] = Double…`——剥离编译**失败**（SAFE 响亮不可编译，第一不变量不违反）；
- **jadx 同败**：storeWrong 输出语法非法 `new String[2][0] = 1;`，storeNumber 标注 `Multi-variable type inference failed`——双方共同缺口，无先例解；
- **潜在 MVP**（无需 LUB）：存入点在值类型不兼容组件时，把数组访问接收者宽化为 `((Object[]) local)[idx] = value;`——任何引用数组可上转 Object[]，源编译恢复且运行时 ASE 保留（不重建源声明型）。

## 处置

登记为**候选窄片**（协变存入宽化，落点=数组元素存入呈现的类型检查；与 heterogeneous-array-init 片相邻但非同落点：那是初始化器元素-组件判据，这是局部声明窄化+存入接收者）。不混片；排在 critical 片后。
