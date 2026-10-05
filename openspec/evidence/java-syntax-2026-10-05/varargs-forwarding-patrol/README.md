# varargs 转发族巡查（2026-10-05 root，负结果）

## 探针

[fixture/VR.java](fixture/VR.java)（`--release 8`）：varargs 定义（`int...` + for-each）、**varargs 转发**（`fwd(int... xs){ return sum(xs); }`——args 数组直传）、数组展开调用（`sum(arr)`）、固定+varargs 混合、零 varargs 实参（`mixed(3)`——空数组）、调用点 `(String[]) make()` 展开形。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- varargs 签名（`int... arg0`）、数组直传转发、`(String[])` cast 展开调用、零实参（编译为空数组）全部恢复；
- 行为 `6/9/6/4/8/3` 逐行 IDENTICAL。

## 处置

负结果归档，不立 spec。varargs 族（定义/转发/展开/混合/零参）确认覆盖。
