# artifact-snapshots Specification

## Purpose
为 JVM artifact 分析提供稳定的物理字节和定位信息，使同名、同内容、多位置的 entry 均可独立追溯，并在资源受限时明确说明未处理的范围。

## Requirements

### Requirement: Immutable physical snapshots
系统 SHALL 接受单独 CLASS 及 ZIP 容器中的 JAR/WAR，建立有输入字节上限的不可变快照；后续读取 MUST 不受原路径修改影响。

#### Scenario: Source changes after open
- **WHEN** 打开文件后原路径被替换
- **THEN** 已打开快照继续返回打开时的字节和身份

### Requirement: Physical entries retain distinct identities
系统 SHALL 按中央目录 ordinal 保留全部物理 entry、raw name、压缩方式和定位信息；重复名称 SHALL 生成诊断而不覆盖前项。

#### Scenario: Duplicate names and MR variants
- **WHEN** JAR 含同名 entry 及 META-INF/versions 下的类
- **THEN** 枚举返回各自不同的物理 ID，且不应用 runtime 过滤

### Requirement: Bounded archive reading
系统 SHALL 有界读取 STORED/DEFLATED entry 并校验 CRC/size；加密、不支持的压缩、非法边界、超限和取消 MUST 显式返回错误或部分状态。系统 MUST 不向用户路径解包或执行 artifact。

#### Scenario: Decompression limit
- **WHEN** entry 声明或实际展开字节超过预算
- **THEN** 读取终止并返回命名的预算维度，不返回空内容冒充成功

#### Scenario: Nested archive locator
- **WHEN** WAR 含 WEB-INF/lib/library.jar
- **THEN** P0 枚举保留该 entry，并声明嵌套内容尚未递归扫描

### Requirement: Signature presence is not trust verification
系统 SHALL 区分 JAR 签名相关 metadata 的存在和签名已验证状态；没有执行签名校验时 MUST 标记 not_verified，不推断 artifact 可信。

#### Scenario: Signed JAR metadata
- **WHEN** 归档包含签名文件但当前请求未执行签名验证
- **THEN** 系统保留 metadata 来源并返回 not_verified，不标记 verified/trusted
