# 普通 try List 与整数前向 carry：最终本地验收

完整类源码恢复已按原输入、原 Runner 验证，未裁剪方法或手改生成 Java。产品提交、自身 CI 与最终 clean-main 交付仍开放；本归档不宣布整个 CF11 或 71/612 账本完成。

后续状态：本归档对应的f014产品已提交推送。追加工厂receiver整数carry输入在v14发现可编译但运行错码；这里的历史绿结果只覆盖下述冻结输入，不能作为当前最终交付。新修复、focused-v17及完整dependency恢复对照另行记录，最终矩阵/双seed/新产品自身CI仍开放。

- frozen CLI：59537768 bytes，SHA `0a30cf2cef1974006105e141789d5a9a8116497fae12d20479b2644d3a0ad2d7`。focused-v14 为629/0/2；420 Java库、178 reader库、原ICN负例及公开List测试通过，fmt/CI exact Clippy/OpenSpec1.11 strict通过。
- matrix-v3：16个完整Jarde cells编译运行成功；baseline17方法/45行、admission13方法/33行同原oracle。JADX baseline8编译失败/no-runtime、admission8成功/33行均按实情保留。root主审计accepted/errors0。
- forward-order-v3：两JDK/default-all四完整16方法cells，64个方法source maps、12行时序oracle一致；覆盖可变字段旧值、producer/test/arm顺序、null arraylength与RHS抛RuntimeException/Error。javac23仅有原source/target8 options警告，runtime stderr空。
- ICN：两编译器/default-all四对v6/v13完整正文不变。真实Ref receiver满足旧同值carry前提，仍交给既有construction/concat完整链证明；新入口限定物理Value::Int。BI两编译器及历史class/default-all六对正文、quality、map、身份不变。
- fresh workspace-v8：195命令/390raw，两个seed各365targets/96batches；合计7124/0/196。每target由Cargo metadata/compiler-artifact/stderr Running/stdout named outcomes独立关联，required gates逐名验证。
- CI额外本地门：六个原selectors的ignored命令共9测试实际通过；两个examples及feature/normal/layered closure检查通过。使用Java25.0.4.1 runtime与钉定JDK23 javac/javap，不能冒充GitHub的完整Temurin25.0.4+7工具链。

v5及focused-v12的真实ICN回归、v6的旧fixture population失败、v7缺失进程句柄的中断收据保留，均未拼成成功。精确新增census为4class/62Code/18handler/145normal-branch/0jsr；仅更新cfg(test)固定tuple与注释。v13/v14 CLI逐字相同，生产reader前缀逐字相同，因此完整运行证据绑定最终二进制，未重复无变化的矩阵。

全部本地运行保持默认Rust stack及5GiB/target1GiB/1秒进程组守卫；最终exact cargo clean成功，target absent。4876来源闭包前后恒同。纯goto等既有map缺口另记，不扩大本片完成范围。

[capture.tar.gz](capture.tar.gz) 12920303 bytes / SHA `7d5cf9f04b83f8e4c54d078c7b10b07a65376023a291506c36c3543f5eb6d36a`。root独立复开1642个payload成员，并将9个final-source成员逐一绑定focused-v14/workspace-v8编译源码和当前文件；见[archive-review.json](archive-review.json)。大JDK/JADX/CLI安装内容未入归档，保留工具与二进制pins。
