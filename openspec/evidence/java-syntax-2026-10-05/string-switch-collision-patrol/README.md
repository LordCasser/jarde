# String switch hashCode 碰撞巡查（2026-10-05 root，负结果——经典边角全过）

## 探针

[fixture/SC.java](fixture/SC.java)（`--release 8`）：`"1012"`/`"14669600"` 为**hashCode 碰撞对**（皆 1507456——brute-force 搜索确认）——javac 对碰撞 case 必须发射**二次 equals 检查**（lookupswitch 到共享桶后 equals 链分辨），加上无碰撞对照 case。

## 结果：**健康，无缺口**（quotes=0 / notrec=0）

- 碰撞对呈现为两个独立 `case "1012":`/`case "14669600":`——**二次 equals 检查机械被正确折回** case 形（不泄漏共享桶内部结构）；
- 行为 `a/b/c/?` 逐行 IDENTICAL——碰撞对精确区分（"1012"→a、"14669600"→b，不串味）。

## 处置

负结果归档，不立 spec。String switch 至此全覆盖（普通/碰撞/枚举经 $SwitchMap）。
