#!/usr/bin/env python3
"""Derive S0 receipts and report from frozen inputs and raw runs, without product execution."""
from collections import Counter, defaultdict
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

spec=importlib.util.spec_from_file_location('baseline',Path(__file__).with_name('s0_baseline.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
OUT=b.CHANGE/'results/s0-resumed-v2'
OLD=b.CHANGE/'results/s0-baseline-v1'

def load(p):return json.loads(p.read_text())
def write(p,x):b.write(p,x)
def pin(p):return b.pin(p)

def main():
    resource=load(OUT/'resource-summary.json')
    assert resource['processes']==resource['expected']==470
    assert resource['collection_complete'], 'collect all repetitions before reporting a finished collection'
    original=load(OLD/'manifest.json')
    census=load(OUT/'census/census-summary.json')
    binding=load(OUT/'census/runtime-binding-supplement.json')
    cases=load(b.FIX/'matrix.json')['cases']+load(b.CHANGE/'fixtures/s0-mechanisms-v2/matrix.json')['cases']
    rows=load(OLD/'matrix-results.json')+load(OUT/'expanded-matrix/matrix-results.json')
    outcomes={};by_family={}
    for tool in ['jarde','jadx']:
        compiled=[r for r in rows if r['tools'][tool].get('compile_exit')==0]
        same=[r for r in compiled if len(r['tools'][tool].get('runtimes',{}))==2 and all(v['exit_code']==0 and v['equal_to_original'] for v in r['tools'][tool]['runtimes'].values())]
        diff=[r for r in compiled if r not in same]
        outcomes[tool]=dict(artifact_configs=len(rows),compiled=len(compiled),both_runtimes_equal=len(same),compiled_but_mismatch=len(diff),
            mismatch_ids=[dict(id=r['id'],compiler=r['compiler'],debug=r['debug']) for r in diff])
    for family in sorted({r['family'] for r in rows}):
        fs=[r for r in rows if r['family']==family];by_family[family]=dict(configs=len(fs))
        for tool in ['jarde','jadx']:
            compiled=[r for r in fs if r['tools'][tool].get('compile_exit')==0]
            by_family[family][tool]=dict(compiled=len(compiled),both_equal=sum(len(r['tools'][tool].get('runtimes',{}))==2 and all(v['equal_to_original'] for v in r['tools'][tool]['runtimes'].values()) for r in compiled))
    write(OUT/'mechanism-results-summary.json',dict(source_cases=len(cases),lineages=len({c['lineage'] for c in cases}),holdout_lineages=8,
        artifact_configs=len(rows),tools=outcomes,by_family=by_family,warning='compiler/debug and A/B variants are correlated; no statistical independence or package-wide success rate is claimed'))
    failed={}
    for label in ['bcprov','s2-009']:
        index=[json.loads(s) for s in gzip.open(OUT/'census'/f'{label}-methods.jsonl.gz','rt')]
        tags=Counter();codes=Counter();first=Counter();stages=Counter()
        for r in index:
            first[(r['first_diagnostic'] or {}).get('code','no_diagnostic')]+=1
            if r['recovery_quality']=='fallback':
                tags.update(r['overlapping_labels']);codes.update(r['diagnostic_codes'])
            stages.update((k,v.get('kind','unknown')) for k,v in r['later_stages'].items())
        failed[label]=dict(physical=census['inputs'][label],binding=binding['method_binding_denominator'][label],
            delivery=load(OLD/'bulk'/label/'summary.json')['counts'],
            fallback_diagnostic_methods=dict(codes.most_common()),fallback_overlapping_heuristic_tags=dict(tags),
            first_reported_diagnostic_methods=dict(first),stage_states=[dict(stage=k,state=s,methods=n) for (k,s),n in sorted(stages.items())],
            semantics='first reported diagnostic is not a proved first blocker; tags overlap and are heuristic; stage states, binding, class compilation and behavior are separate observations')
    write(OUT/'failure-distribution.json',failed)
    # Complete declarations are inputs here; the current product's abbreviated identity is not substituted for them.
    environments={}
    for label in ['bcprov','s2-009']:
        with gzip.open(OLD/'bulk'/label/'records.jsonl.gz','rt') as f:
            first=next(x for s in f if (x:=json.loads(s))['kind']=='method')
        environments[label]=dict(effective_identity_observed=first['delivery']['analysis']['environment_identity'],
            declared_loader_count=1,declared_provider_count=0,parent=None,delegation='parent_first',module_mode='class_path',
            roots=load(OLD/'inventory'/f'{label}.roots.json'),external_override='none',runtime_transformation='none',
            implicit_platform_headers=False,source_target=dict(java_release=8,independent_configuration_available=False,alias='RuntimeProfile.java_release'))
    sources=[pin(p) for folder in [b.CHANGE/'fixtures/s0-mechanisms-v1',b.CHANGE/'fixtures/s0-mechanisms-v2'] for p in sorted(folder.rglob('*')) if p.is_file()]
    freeze=dict(state='input_and_matrix_frozen_resource_gate_incomplete',engine_commit=original['engine_commit'],
        git_tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=b.ROOT,text=True).strip(),
        authorization='本会话用户要求对当前项目既有本地 JAR/WAR 建立 S0 基线；仅分析原件。该记录不声明重新分发第三方原件的许可。',
        original_manifest=pin(OLD/'manifest.json'),artifacts=original['artifacts'],controls=load(OLD/'controls/manifest.json'),
        control_scope_correction=pin(OUT/'controls/summary.json'),compilers=original['jdks'],
        jdk8_compiler_tools=pin(b.J8/'lib/tools.jar'),jadx=original['jadx'],environments=environments,
        effective_bulk_limits={label:load(OLD/'bulk'/label/'header.json')['limits'] for label in ['bcprov','s2-009']},
        compiler_flags=dict(javac8=['-source','8','-target','8'],javac23=['--release','8'],common=['-proc:none','-sourcepath empty','-classpath explicitly declared','-g:none or -g']),
        fixture_sources=sources,resource_plan=pin(OUT/'resource-plan.json'),resource_binary=load(OUT/'resource-binary.json'),
        cli=original['cli'],negative_build=pin(OUT/'negative-build/build-receipt.json'),resource_build=pin(OUT/'build-r3/build-receipt.json'),
        never_executed='真实 JAR/WAR 的目标代码从未执行；只有本 change 明确选择的受控源码 fixture 运行 oracle。',
        excluded_materials=['v1 performance 19/110 interrupted processes','v1 nested/duplicate default snapshot_all listing','v2 initial unexecuted resource plan','v2 failed adapter build and preliminary smokes','v2 resource-interrupted-r1 40/470 processes; RSS guard followed by collector print error'])
    write(OUT/'input-freeze.json',freeze)
    guards=[dict(config=c['id'],failures=c['guarded_failures']) for c in resource['configs'] if c['guarded_failures']]
    gaps=['现有端口没有实际 IR 构建开始/完成计数；Body/recovery 次数和 ir_items 计费不能代替。',
          'live facts/handles、候选/中间 AST 的实际独立峰值没有现成 observer；cache、pending result weight 和进程 RSS 不能代替。',
          'ReadBytes 是计费读量，未取得独立的 OS read 字节观测；原始 time -l 的 block operations 仅表示该 OS 指标。']
    faults=[]
    plan=load(OUT/'resource-plan.json')
    raw=load(OUT/'resource-raw.json')
    for cfg in plan['configs']:
        if cfg['fault']=='normal':continue
        a=[r for r in raw if r['config_id']==cfg['id']]
        details=[r['samples'][0]['sample']['details'] for r in a if r['samples']]
        faults.append(dict(fault=cfg['fault'],id=cfg['id'],samples=len(a),valid_returns=len(details),
            missing_prepared=[d.get('missing_prepared') for d in details],
            workers_alive_after_return=[d.get('workers_alive_after_return') for d in details],
            token=[d.get('token') for d in details if d.get('token')]))
    write(OUT/'failure-path-results.json',faults)
    e2=[f for f in faults if f['fault']=='panic'][0]
    acceptance=dict(S0='not_accepted',completed_tasks=['1.1','1.2','1.3','1.4','1.6'],remaining_tasks=['1.5','1.7'],
        gates={'A0.1':'accepted as frozen measurement inputs and denominators','A0.2':'accepted as baseline matrix/oracle inputs; product failures are not recoveries','A0.3':'not accepted: required observability fields missing'},
        measurement_collection_complete=resource['collection_complete'],guarded_product_failures=guards,measurement_gaps=gaps,
        e2_panic_missing_prepared_reproduced=any(e2['missing_prepared']),S1='not_started',
        scope='未修改 reader/JVM/Java/Engine/CLI 的产品实现；未修改主规格和历史 change 状态。')
    write(OUT/'acceptance.json',acceptance)
    family_lines=[]
    for family,d in by_family.items():
        family_lines.append(f"| {family} | {d['configs']} | {d['jarde']['compiled']} | {d['jarde']['both_equal']} | {d['jadx']['compiled']} | {d['jadx']['both_equal']} |")
    class_results=load(OUT/'real-class-isolated-compile/results.json')['results']
    classes={label:[r for r in class_results if r['artifact']==label] for label in ['bcprov','s2-009']}
    census_lines=[]
    for label in ['bcprov','s2-009']:
        c=census['inputs'][label];z=binding['method_binding_denominator'][label]['cross_tab'];bound=sum(x['methods'] for x in z if x['binding_status']=='bound')
        census_lines.append(f"| {label} | {c['independent_zip_physical_class_entries']:,} | {c['method_records']:,} | {c['independent_zip_code_present_methods']:,} | {c['independent_zip_code_absent_methods']:,} | {bound:,} | {sum(x['methods'] for x in z if x['binding_status']=='explicit_unbound'):,} | {c['raw_facts_stage_states']['completed']:,} |")
    top_lines=[]
    for label in ['bcprov','s2-009']:
        for code,n in list(failed[label]['fallback_diagnostic_methods'].items())[:7]:top_lines.append(f'| {label} | `{code}` | {n:,} |')
    fault_lines=[]
    for f in faults:
        fault_lines.append(f"| {f['fault']} | {f['samples']} | {f['valid_returns']} | {sum(bool(x) for x in f['missing_prepared'])} | {max(f['workers_alive_after_return'],default=0)} |")
    guarded_count=sum(len(x['failures']) for x in guards)
    report=f'''# S0 v2 中间检查点（观测扩展前）

引擎固定为 `{original['engine_commit']}`。本轮完成输入、分母、机制矩阵、隔离 oracle 和停止路径采集，**S0 尚未通过全部验收，S1 未启动**。OpenSpec 当前完成 5/57 项；S0 的 1.5、1.7 保持未完成。原因是 A0.3 要求的若干实际工作/峰值观测没有现成端口，不能用计费量或 RSS 填空。

## 输入怎么固定

原件是 bcprov JAR（`{original['artifacts'][0]['sha256']}`）与 S2-009 WAR（`{original['artifacts'][1]['sha256']}`）。完整路径、字节数、SHA、CLI/观测二进制、JDK 8/23 runtime image、JDK 8 tools.jar、JADX launcher/libs、源码输入与运行环境均在 [input-freeze.json](input-freeze.json)。固定 JADX revision 为 `{original['jadx']['source_commit']}`。

两个真实包都声明单一 `app` loader、parent-first、有序显式 roots、无 parent/provider、无隐式 JDK Header、Java 8、MR disabled、generic layout。WAR 根类使用 `WEB-INF/classes/` prefix，52 个 nested lib 各自声明 root。当前源码目标仍借用 runtime release；这不是已实现独立 SourceTarget。资源计划的实际方法局部预算与总预算分开保存。

旧 nested/duplicate 列表误用了 `snapshot_all`。本轮按 `artifact_tree` 重采：nested 1 类、同字节不同 origin 2 类、MR base/version 2 条物理定义，均 Complete；见 [controls](controls/summary.json)。这只证明物理读取，不冒充 MR runtime 选择或源码目标独立已实现。

## 真实包的分母与失败

| 输入 | 物理 class | 物理 method | 含 Code | 无 Code | 本环境绑定 | 确证遮蔽 | RawFacts 完成/Body 解码结果 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(census_lines)}

独立 Python ZIP/classfile shell 清点与原流逐物理类、成员 ordinal、原始 name/descriptor 对齐，0 个解析错误或身份不匹配；它只做测试清点，不成为第二个生产 parser。NoBody 不进入 Body 解码，但已通过绑定的 NoBody 计入绑定分母。WAR 有 269 个含 Code 和 29 个无 Code 方法被遮蔽，合计 298，涉及 28 个物理类；原流的 `delivery=recovered` 不能把它们算为成功。

这些遮蔽已由 claimed/selected 完整物理身份及同名 reads 核实：commons-collections-3.1 的 72 个方法被更前面的 commons-beanutils 同名定义选中；commons-logging-api-1.1 的 226 个被 commons-logging-1.1.1 选中。逐项证据见 [runtime binding](census/runtime-binding-supplement.json)。物理清单继续保留全部 origin。

bcprov 的 Structured/Fallback 为 10,854/3,641，WAR 为 41,145/12,400；NoBody 分别 508/3,635。含语句分别 11,781/44,693，不等于可编译或行为正确。原 bulk 已遍历全部类，但 final execution 都是 Partial，分别报告 ir_items、analysis_steps 的方法局部停止。`methods_executed` 是 exporter 的记账口径，不能替代实际 Body/IR 工作。

以下只统计 Fallback 方法中出现该诊断的方法数，可重叠：

| 输入 | 已观察诊断 | 方法数 |
| --- | --- | ---: |
{chr(10).join(top_lines)}

首诊断也可能只是名称/呈现警告。上表不能当成互斥根因分区，更不能从某条诊断消失推导恢复完成。原始状态、后续分析阶段、首诊断、全部标签及逐方法物理身份见 [failure-distribution.json](failure-distribution.json) 与 [census](census/census-summary.json)。本轮没有修规则，所以没有“修复后显露的第二阻塞”对照；后续阶段必须接着保存。

真实包按旧算法选 12 类/包：6 个物理身份 SHA 排序样本，再取 6 个 fallback 较多样本，仅限 top-level 名。原生成源码逐字重编，classpath 移除当前目标 `X.class` 与 `X$*.class`，其他依赖保持原顺序。两 compiler 均只有 bcprov 的 LocaleString 和 WAR 的 ognl/NodeType 成功，即各 1/12；其余失败原流保留。没有执行真实包目标，也没有全包编译/行为 oracle。单类样本依然可引用别的原始类家族，不能据此宣称整包重建。见 [完整类重编](real-class-isolated-compile/results.json)。

## 机制矩阵是哪些输入

34 个源码 case、18 个 lineage，派生 136 个 compiler/debug 输入：原 16 个 case 共 64 组；新 18 个 development case 共 72 组。A/B 变形归回 8 个原 dev lineage；两数值 case 是新 dev lineage；8 个 holdout 源码和 driver 的 16 个旧 pin 全部匹配。本轮未用 holdout 修改产品规则，后续若用于调参须标污染。

S2 两族是引用合流/生命周期与泛型传递，S3 是旧值/多消费者与字段数组效果顺序，S4 是 loop 内 catch 与 catch/嵌套出口，S5 是声明/重载与匿名类捕获/初始化。每族增加名称、long 槽布局、消费者、trace 前后语句与结构变形，另有 javac8/javac23 target8 × debug/no-debug。32 个 A 的 Code 均实际变化、max_locals 均增加 2，不能仅凭源码推测变形成立。

| 族 | 产物配置 | Jarde 可编译 | Jarde 双 VM 同原 | JADX 可编译 | JADX 双 VM 同原 |
| --- | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(family_lines)}

合计 Jarde 40/136 可编译、36/136 双 VM 同原；JADX 126/136 可编译、116/136 双 VM 同原。这些是相关样本的有限基线。原 class 是预期，JADX 分歧独立保存。

Jarde 的四组行为不同都来自 R1-dev-B：生成源码的空循环保留显式 `@bytecode` fallback 和 catch 局部作用域拒绝说明，原样编译后少执行循环效果。它是可编译的不完整投影，不是已证明 Structured 的静默错误，也不能因标 Unproven 就计成功。JADX 原 8 组、新 C2-dev-A 无 debug 的 2 组分歧同样保存。

所有 136 个 driver 目录仅含自己的 Driver class；272 个原 class VM 命令使用 `driver:original`，332 个生成 VM 命令只使用 `driver:对应生成 classes`。返回、trace、异常类型逐输出核对。数值样本实际观察窄化、int wrap、NaN raw bits `7fc12345/7fc00000` 与正负零 `00000000/80000000`。见 [独立矩阵复核](matrix-audit/audit.md)、[原矩阵](../s0-baseline-v1/matrix-results.json)、[扩展矩阵](expanded-matrix/matrix-results.json)。

13 个现有拒绝合同测试实际通过，分别覆盖缺失/重复使用约束、LVT 不完整、泛型 facts、旧值消费者、exception edge、boolean/numeric 边界、外来入口/出口/归属、静态泛型初始值及匿名投影第二创建点/外部闭包。它们是 fixture/proof mutation 负例，不是新的 holdout，也不冒充完整类行为 oracle。命令、来源、二进制与结果在 [negatives](negatives/plan.json)。

独立循环 phi 同时复制矩阵、handler 全域、动态加载/反射/未知调用者闭合尚未覆盖；数值样本也不代表全部数值 lowering。ECJ/legacy 保留现有负例与历史回归来源，本轮没有新增完整 ECJ 行为矩阵。这些范围不得在 S2–S5 的验收中被省略或称已验证。

## 资源与失败路径

新批次收齐 470/470 次实验，每配置 10 个独立进程，循环交错顺序。旧 v1 的 19/110 与新首批 40/470 都留为中断材料，没有混进本批。后者在 WAR RSS 守卫触发后因采集器打印空耗时退出；修正采集器后整批重跑，配置、阈值公式和 1 GiB 监测阈值保持不变。

时间门预声明为 `max + max(20% median, 3*MAD, 1000us)`，RSS 为 `max + max(20% median, 3*MAD, 1MiB)`，按相同 shape 比较；usage 固定本 shape 各维度最大值，elapsed 分开。冷热是进程/cache 初态，不声称清空 OS page cache；hot 含单独 warmup。主计时使用适配器操作窗口的单调时钟（含准备调用和结果摘要，不含最终 stdout 序列化），进程 real 与轮询 wrapper 用时分列。

本批有 {guarded_count} 次硬监测触发的失败，详见 [资源汇总](resource-summary.json)；守卫以 0.5 秒轮询进程组 RSS，触发阈值不是 OS 硬内存限制，实际可越过阈值。被终止的请求没有完成耗时或最终 report，指标按 censored/失败保存，不补数、不放宽预算。正常返回样本的时间/MAD、RSS、Body/recovery 端口、cache、pending weight、慢 sink 编码缓冲及停止信息均有原流。

| 路径 | 重复 | 有返回记录 | 缺 Prepared 的样本 | 返回后活 worker 最大值 |
| --- | ---: | ---: | ---: | ---: |
{chr(10).join(fault_lines)}

token 在真实 Prepared 后 1ms 并发置位，保存置位到返回和其后端口计数；hook 是完成计数，不能据此断言取消后一定启动了新 Body。sink Stop、method_bodies 预算、预过期 elapsed、慢 sink、worker panic、准备前 Refused 分开。Refused 无 Prepared 合法；sink 拒绝后不强补事件。panic 是否重现 E2 缺 Prepared：`{acceptance['e2_panic_missing_prepared_reproduced']}`。已测协议失败可作为基线，不能称产品合同通过。见 [失败路径](failure-path-results.json)。

目前阻塞 A0.3 的缺测是：

{chr(10).join('- '+g for g in gaps)}

原计划明确“缺测不通过”。所以 1.5 不能勾选，依赖它的 1.7 也不能完成；470 次采集完成不能替代该合同。资源文件区分 collection_complete 与成功返回/完成的请求，不把被守卫终止的实验当成功。

## 后续阶段怎样接这些结果

| 承接 | 输入和门 |
| --- | --- |
| S1 | 保留完整环境声明与错误绑定样本、Header/Body 端口、evidence/cache 配置；A0.3 补齐前不得跳过 S0 |
| S2 | T1/T2、数值/boolean 及拒绝合同；新增 phi/DA 域须给独立输入和原 class 对照，不能只改旧期望 |
| S3 | E1/E2、numeric、异常/效果顺序；拒绝糖化后保留有效顺序表示，对可编译 fallback 仍核对行为 |
| S4 | R1/R2 的组合及 R1-dev-B 拒绝来源，沿 S2 变量合同与区域归属解决；不按名字特判该输入 |
| S5 | C1/C2 与当前目标家族隔离编译、开放使用闭包负例；不能把 2/24 当整包重建率 |
| S6 | 独立 query 的零 Body 基线、重复请求、慢 sink、token 和 E2；组合不得掩盖各阶段 coverage |

E1 计费/成本观测涉及资源门；E2 bulk 关闭在 panic 路径承接；E3 格式/绑定版本随所属阶段；E4 parser 仅有具体阻塞再评估；E5 机械拆分无收益门；E6 历史 observed_counts/goto mapping 不沿用错误计数。本轮既有 13 项负例与输入证据不改变旧 change 状态，没有同步主规格或归档。

继续完成 S0 需要先补观测范围：实际 read/IR 工作点、live fact/候选分组峰值，以及端口开关不改变正文/身份/停止的对照。优先扩展现有 test-support 端口，单位和生命周期先定义；不引入公共报告字段、跨请求缓存或通用 observer 框架。这会触及 reader/JVM/Java 的观测代码，超出当前仅用适配工具采集的范围，需要先修订 S0 实施任务；本轮未把该实现塞入 S1。

完整机器验收状态见 [acceptance.json](acceptance.json)。
'''
    (OUT/'README.md').write_text(report)
    print(json.dumps(dict(tasks=acceptance['completed_tasks'],gates=acceptance['gates'],resources=resource['processes'],guarded=guarded_count),ensure_ascii=False,indent=2))

def observed():
    """Accept the declared observation domain; retain censored product failures as failures."""
    import sys
    from statistics import median
    out=b.CHANGE/'results/s0-observed-v3'
    plan=load(out/'resource-plan.json'); summary=load(out/'resource-summary.json')
    rows=load(out/'resource-raw.json'); prior=load(OUT/'resource-raw.json')
    configs={c['id']:c for c in plan['configs']}
    samples=[(r,r['samples'][0]) for r in rows if r['exit_code']==0 and len(r['samples'])==1 and r['guard'] is None]
    guards=[r for r in rows if r['guard'] is not None]
    def clean(v):
        if isinstance(v,dict):return {k:clean(x) for k,x in v.items() if k!='elapsed_millis'}
        if isinstance(v,list):return [clean(x) for x in v]
        return v
    def core(s,cfg):
        d=s['sample']['details'];r=dict(snapshot=s['snapshot'],digest=s['sample']['semantic_text_digest'],text_bytes=s['sample']['text_bytes'])
        if cfg['shape'] in ['single','class','query']:r['details']=clean(d)
        else:
            report=d['report'];z=report['summary']
            r['details']=dict(stop=report['stop'],execution=clean(z['execution']),summary={k:v for k,v in z.items() if k.startswith(('classes_','methods_'))},final_delivered=report['final_delivered'],offered_methods=d['offered_methods'])
        return r
    normal=[]
    for raw,s in samples:
        cfg=configs[raw['config_id']]
        if cfg['fault']!='normal':continue
        refs=[x['samples'][0] for x in prior if x['config_id']==cfg['id'] and x['exit_code']==0 and len(x['samples'])==1]
        normal.append(dict(id=cfg['id'],repeat=raw['repeat'],core_equal_v2=any(core(s,cfg)==core(x,cfg) for x in refs),
            billed_usage_equal_v2=any(clean(s['sample']['usage'])==clean(x['sample']['usage']) for x in refs),
            facade_counts_equal_v2=any(s['sample']['counts']==x['sample']['counts'] for x in refs)))
    write(out/'normal-comparison.json',normal)
    source=load(out/'observation-source-manifest.json')
    source_checks=[dict(**x,matches=pin(Path(x['path']))['sha256']==x['sha256']) for x in source['files']]
    checks={
        'collection_complete':summary['collection_complete'],
        'all_config_repeat_pairs':len(rows)==len(configs)*plan['repeats'] and len({(r['config_id'],r['repeat']) for r in rows})==len(rows),
        'same_configs_as_v2':plan['configs']==load(OUT/'resource-plan.json')['configs'],
        'normal_core_usage_counts_equal':all(all(v for k,v in x.items() if k not in ['id','repeat']) for x in normal),
        'all_returned_samples_have_ports':all('observed_work' in s['sample'] and 'retention' in s['sample'] for _,s in samples),
        'artifact_file_bytes_match':all(s['open_actual_reads']['file_read_bytes']==configs[r['config_id']]['input_pin']['bytes'] for r,s in samples),
        'query_zero_body_ir_ast':all(s['sample']['observed_work']['reader']['body_decode_attempts']==0 and s['sample']['observed_work']['jvm']['starts']==[0]*6 and s['sample']['retention']['java_end']['expr_peak']==s['sample']['retention']['java_end']['stmt_peak']==0 for r,s in samples if configs[r['config_id']]['shape']=='query'),
        'java_owned_nodes_and_candidates_released':all(all(v==0 for k,v in s['sample']['retention']['java_end'].items() if k.endswith('_live')) for _,s in samples),
        'reader_typed_handles_released':all(s['sample']['retention']['reader_end']['prepared_handles_live']==s['sample']['retention']['reader_end']['container_handles_live']==0 for _,s in samples),
        'no_workers_after_return':all(s['sample']['details'].get('workers_alive_after_return',0)==0 for _,s in samples),
        'censored_runs_have_observation_prefix':all(r['guard']=='rss' and r['observation_prefix'] and all(x['final'] is False for x in r['observation_prefix']) for r in guards),
        'sources_match_frozen_overlay':all(x['matches'] for x in source_checks),
        'resource_binary_matches':pin(Path(load(out/'resource-binary.json')['path']))==load(out/'resource-binary.json'),
    }
    fixture_equivalence={}
    for shape in ['single','class','query','bulk']:
        digests={s['sample']['semantic_text_digest'] for r,s in samples if (c:=configs[r['config_id']])['label']=='fixture' and c['fault']=='normal' and c['shape']==shape}
        fixture_equivalence[shape]=sorted(digests)
    cache_equivalence={}
    for shape in ['single','class','query','bulk']:
        for evidence in ['essential','all']:
            cache_equivalence[shape+'/'+evidence]=sorted({s['sample']['semantic_text_digest'] for r,s in samples if (c:=configs[r['config_id']])['label']=='fixture' and c['fault']=='normal' and c['shape']==shape and c['evidence']==evidence})
    checks['cache_paths_core_equal']=all(len(v)==1 for v in cache_equivalence.values())
    evidence_replay=load(out/'evidence-selection/summary.json')
    product_checks=dict(optional_evidence_core_equal=all(len(v)==1 for v in fixture_equivalence.values()),class_source_evidence_equal=evidence_replay['equal'])
    checks['evidence_difference_independently_replayed']=not evidence_replay['equal'] and all(x['exit_code']==0 for x in evidence_replay['rows']) and (out/'evidence-selection/source.diff').stat().st_size>0
    paired=load(out/'evidence-selection/compile-oracle.json')['rows']
    essential=[r for r in paired if r['evidence']=='essential']
    full=[r for r in paired if r['evidence']=='all']
    checks['evidence_compile_and_oracle_pair_verified']=len(paired)==4 and {r['compiler'] for r in essential}=={r['compiler'] for r in full}=={'javac8','javac23'} and all(r['compile_exit']!=0 and not r['runtimes'] for r in essential) and all(r['compile_exit']==0 and set(r['runtimes'])=={'jvm8','jvm23'} and all(v['exit_code']==0 and v['equal_to_original'] for v in r['runtimes'].values()) for r in full)
    map_only=load(out/'evidence-selection/source-map-only-summary.json')
    checks['source_map_only_replays_all_source']=map_only['exit_code']==0 and map_only['equals_all'] and (out/'evidence-selection/source-map-only.java').read_bytes()==(out/'evidence-selection/all.java').read_bytes()
    single=[s['sample'] for r,s in samples if r['config_id']==plan['configs'][2]['id']]
    class_samples=[s['sample'] for r,s in samples if r['config_id']==plan['configs'][10]['id']]
    checks['positive_body_and_ir_ports']=len(single)==10 and all(s['observed_work']['reader']['body_decode_attempts']==1 and s['observed_work']['jvm']['starts']==s['observed_work']['jvm']['completions']==[1]*6 and s['retention']['java_end']['expr_peak']>0 for s in single)
    checks['positive_class_candidate_and_ast_ports']=len(class_samples)==10 and all(s['retention']['java_end']['class_candidate_payloads_peak']>0 and s['retention']['java_end']['method_ast_sources_peak']>0 for s in class_samples)
    test_rows=[r for r in load(out/'checks/results.json') if r['id']!='lifetime']+load(out/'checks-r2/results.json')
    checks['focused_tests_pass']=all(r['exit_code']==0 for r in test_rows) and any(r['id']=='lifetime' for r in test_rows)
    checks['focused_tests_executed']=all('1 passed; 0 failed' in (out/('checks-r2' if r['id']=='lifetime' or r['id'].startswith('generic-') else 'checks')/(r['id']+'.stdout')).read_text() for r in test_rows)
    checks['token_trigger_during_request']=all(s['sample']['details']['token'] and s['sample']['details']['token']['set_before_return'] for r,s in samples if configs[r['config_id']]['fault']=='token')
    checks['ir_starts_cover_publications']=all(all(a>=z for a,z in zip(s['sample']['observed_work']['jvm']['starts'],s['sample']['observed_work']['jvm']['completions'])) for _,s in samples)
    builds=[load(out/'build-r5/build-receipt.json'),load(out/'lifetime-build-r2/build-receipt.json')]
    checks['builds_pass_and_targets_removed']=all(not Path(x['target']).exists() and x['target_absent'] and all(c['exit_code']==0 and c['stop'] is None for c in x['commands']) for x in builds)
    verification=dict(checks=checks,normal_comparisons=len(normal),returned_samples=len(samples),guarded_failures=len(guards),focused_tests=len(test_rows),
        source_checks=source_checks,fixture_core_equivalence=fixture_equivalence,cache_core_equivalence=cache_equivalence,product_checks=product_checks,tests=test_rows)
    write(out/'verification.json',verification)
    assert all(checks.values()),checks
    faults=[]
    for cfg in plan['configs']:
        if cfg['fault']=='normal':continue
        ds=[s['sample']['details'] for r,s in samples if r['config_id']==cfg['id']]
        faults.append(dict(id=cfg['id'],fault=cfg['fault'],returns=len(ds),missing_prepared=[d.get('missing_prepared') for d in ds],
            worker_alive=[d['workers_alive_after_return'] for d in ds],tokens=[d['token'] for d in ds if d.get('token')]))
    write(out/'failure-path-results.json',faults)
    missing_prepared=any(any(d['missing_prepared']) for d in faults if d['fault']=='panic')
    acceptance=dict(S0='accepted_measurement_baseline',gates={'A0.1':'accepted: fixed inputs and separate physical/binding/delivery denominators','A0.2':'accepted: frozen mechanism and isolated oracle inputs, including known failures','A0.3':'accepted: declared object-count/work ports and ten interleaved repeats, censored product failures preserved'},
        S1='not_started_user_gate',product_contracts_passed=False,completed_tasks=['1.1','1.2','1.3','1.4','1.5','1.6','1.7','1.8','1.9'],
        scope='S0 measurements only; observer fields/hooks do not implement recovery, resolver, source-target or query changes',
        product_failures=dict(rss_guarded=len(guards),panic_missing_prepared_reproduced=missing_prepared,class_source_depends_on_optional_evidence=not evidence_replay['equal']),
        boundaries=['Object counts are not heap-byte attribution or all raw Arc references.','Region trees and unlisted local proof records are outside typed retention counts; RSS remains the aggregate memory measurement.','Censored snapshots give lower-bound peaks; no completed latency/RSS threshold is invented for a killed request.','Real-class compilation is a 24-class sample; no real target execution or whole-package rebuild claim.'])
    write(out/'acceptance.json',acceptance)
    freeze=dict(recovery_baseline=pin(OUT/'input-freeze.json'),recovery_commit=source['base_head'],observer_overlay=pin(out/'observation-source-manifest.json'),
        observer_sources=pin(out/'observation-sources.tar.gz'),tracked_patch=pin(out/'observation.patch'),resource_plan=pin(out/'resource-plan.json'),
        resource_binary=load(out/'resource-binary.json'),builds=[pin(out/'build-r5/build-receipt.json'),pin(out/'lifetime-build-r2/build-receipt.json')],
        measurement_tools=[pin(p) for p in sorted((b.CHANGE/'tools').glob('*')) if p.is_file()],
        note='Recovery inventory/oracles use the pinned original CLI. Resource runs use that HEAD plus the observer-only overlay, validated against v2. Temporary binary paths may not survive migration; rebuild from pinned sources/commands.')
    write(out/'input-freeze.json',freeze)
    def span(values):
        a,z=min(values),max(values);return str(a) if a==z else f'{a}–{z}'
    resource_lines=[];work_lines=[]
    for index in [2,10,18,26,35,39]:
        cfg=plan['configs'][index];z=next(x for x in summary['configs'] if x['id']==cfg['id'])
        resource_lines.append(f"| {cfg['label']} / {cfg['shape']} | {z['valid']}/10 | {z.get('request_micros',{}).get('median','截断')} | {round(z['rss_bytes']['max']/(1<<20),2) if z.get('rss_bytes') else '截断'} | {len(z['guarded_failures'])} |")
        ss=[s['sample'] for r,s in samples if r['config_id']==cfg['id']]
        if ss:
            cells=[span([x['observed_work']['reader'][k] for x in ss]) for k in ['read_bytes','body_decode_attempts']]
            cells += [span([x['observed_work']['jvm']['starts'][i] for x in ss]) for i in [1,4,5]]
            cells += [span([x['retention']['reader_end']['class_facts_peak'] for x in ss])]
            cells += [span([x['retention']['java_end'][k] for x in ss]) for k in ['class_candidate_payloads_peak','expr_peak','stmt_peak']]
            work_lines.append('| '+cfg['label']+' / '+cfg['shape']+' | '+' | '.join(cells)+' |')
    fault_lines=[f"| {d['fault']} | {d['returns']} | {sum(bool(x) for x in d['missing_prepared'])} | {max(d['worker_alive'],default=0)} |" for d in faults]
    text=f'''# S0 冻结结果

S0 的 A0.1–A0.3 已按声明测量域验收，9 项任务完成。**S1 未启动。**这表示输入、失败分布、机制矩阵和资源门已冻结；当前产品的恢复和资源合同仍有失败。

## 先看恢复结果

真实包原件、CLI/JDK/JADX、loader/provider/roots、预算和源码目标别名限制见 [输入冻结](input-freeze.json) 与 [v2 输入声明](../s0-resumed-v2/input-freeze.json)。恢复引擎基线为 `{source['base_head']}`；新资源批次叠加独立保存的 test-support 观测源码，未修改恢复规则、预算判断、主规格或旧 change 状态。

| 输入 | 物理 class | 物理 method | 含 Code | 无 Code | 确证遮蔽方法 |
| --- | ---: | ---: | ---: | ---: | ---: |
| bcprov | 2,430 | 15,003 | 14,495 | 508 | 0 |
| S2-009 WAR | 7,200 | 57,180 | 53,516 | 3,664 | 298 |

真实包恢复分布来自固定原 CLI 的完整原流，新观测批次的 WAR bulk 被 RSS 守卫截断，两种运行分母分开。bcprov 的 Structured/Fallback 为 10,854/3,641；WAR 为 41,145/12,400。它们都不等于可编译或行为一致，原 final execution 也均为 Partial。WAR 的 298 个遮蔽方法包括 269 个含 Code、29 个无 Code；原流写 `recovered` 不构成成功证据。见 [逐方法失败分布](../s0-resumed-v2/failure-distribution.json)、[绑定核实](../s0-resumed-v2/census/runtime-binding-supplement.json)。

Fallback 高频诊断集中在字段未发射、区域未覆盖、分支不合流和对象创建效果交错。诊断重叠且首条可能是警告，不作为互斥根因。完整类采用固定算法抽样，每次移除自身 X/X$ 家族的原 class 后原样重编，当前仅 **2/24** 个选中类在两个编译器下均通过；不外推整包重建率。见 [隔离编译](../s0-resumed-v2/real-class-isolated-compile/results.json)。

受控矩阵共有 34 个源码案例、18 个 lineage、136 个 compiler/debug 配置，保留集为 8 个 lineage；A/B 改名/消费者变形没有充当独立保留集。Jarde **40/136 编译，36/136 双 runtime 行为一致**；JADX **126/136 编译，116/136 双 runtime 行为一致**。Jarde 4 组可编译但不一致均保留显式 bytecode fallback，不计恢复成功。原 class 是 oracle；JADX 的 10 组分歧单列。见 [机制结果](../s0-resumed-v2/mechanism-results-summary.json)、[完整源码与隔离执行审计](../s0-resumed-v2/matrix-audit/audit.md)。真实 JAR/WAR 的目标代码未执行。

## 本次补齐的观测

| 端口 | 实际口径 |
| --- | --- |
| artifact 文件 | FileSnapshotSource 的 read 调用、返回字节、EOF/错误；单列 open |
| ZIP 条目 | wrapped reader 调用和输出字节；Deflate 为解压输出，Stored 为条目字节；不是 OS 物理磁盘 I/O |
| Body / IR | 共享 Code decoder 实际入口；pipeline、raw CFG、contexts、canonical CFG、frames、SSA 的实际调用与表发布 |
| facts / handles | ClassFacts、ContainerFacts payload 对象；PreparedClass、ContainerFactsHandle 值的 live/peak；Arc 共享不重复计 payload |
| Java | Expr/Stmt 节点、列明的类级候选 payload、保留 AST source 的 live/peak；深 clone 计新对象，drop 回落 |
| 总量 / 交付 | RSS、cache 权重、bulk 窗口权重、文本/编码缓冲分别保留，不相加冒充总内存 |

对象峰值的单位是**对象数**，不声称精确堆字节或全部 Arc 引用。候选清单与未覆盖的局部 proof/Region 树在源码说明中；RSS 继续反映总量。观测用固定原子，不持有产品，normal build 的字段/状态由 cfg 移除。hot warmup 后只在静止点 reset peaks，工作量读取单调差值。

最终 {len(test_rows)} 项聚焦验证通过，覆盖真实文件/归档读取、cache/Arc 与值 clone、句柄释放、4 worker 同时持有 AST 的峰值及既有拒绝/停止合同。初轮零预算测试失败原流保留，修正测试限额后独立重跑。普通/观测构建、Java/root 单测目标均编译通过，构建 target 已精确删除。见 [验证](verification.json)、[构建](build-r5/build-receipt.json)、[生命周期重跑](checks-r2/results.json)。

## 资源批次与停止

正式采集 **{len(rows)}/{len(configs)*plan['repeats']}** 次：{len(samples)} 次有返回、{len(guards)} 次触发 RSS 守卫。每配置 10 个独立进程，循环交错；预检和旧中断批次不混入。{len(normal)} 个正式正常返回请求与 v2 的正文/身份/核心状态、逻辑计费及 facade 计数一致。同一 evidence 下 cache 禁用/冷/热/容量不足的正文一致，结构 query 实测零 Body/IR/AST。见 [协议](resource-plan.json)、[资源汇总](resource-summary.json)、[逐次对照](normal-comparison.json)。

| 负载（cold/essential） | 返回 | 操作窗口中位数 µs | 返回进程 RSS 最大 MiB | 守卫失败 |
| --- | ---: | ---: | ---: | ---: |
{chr(10).join(resource_lines)}

操作窗口包含适配器的预算/调用准备和返回摘要，不含最终 stdout 序列化；open、进程 wall 与 wrapper 时间分开。阈值先声明：时间 `max + max(20% median, 3*MAD, 1000us)`；RSS `max + max(20% median, 3*MAD, 1MiB)`；usage 按 shape 取各维度最大值，elapsed 单列。OS page cache 没有清空。被杀请求没有完成阈值可推导，1 GiB RSS 监测门继续固定，不提高上限使其通过。

| 负载 | 条目输出字节 | Body 入口 | raw CFG 调用 | frames 调用 | SSA 调用 | ClassFacts 峰值 | 类候选峰值 | Expr 峰值 | Stmt 峰值 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
{chr(10).join(work_lines)}

本轮另确认**可选证据改变类族恢复**：E2 输入在 essential 下使用 `E2FieldArrayOrderDev$Box` 且没有嵌套类声明，all 下改用 `Box` 并发射完整嵌套类；Body 入口由 7 增为 8。固定旧 CLI 已独立复现，见 [完整源码差异](evidence-selection/source.diff)、[essential](evidence-selection/essential.java)、[all](evidence-selection/all.java)。v2 与 v3 各 cache 路径、各 10 次均稳定呈现这一差异，因此不是新增观测造成。A0.3 将它冻结为产品失败，**A1.1/S1 的 2.2 必须修复**，同时给 S5 类族组装门保留反例；没有把它改成可接受的 evidence 语义。

源码不修改且编译路径隔离时，essential 在 javac 8/23 都缺 Box 而失败；all 在两个编译器通过，两个产物分别在 JVM 8/23 执行，四次均与原始受控 class 一致。仅请求 source_map 的正文也与 all 完全相同。源码追踪确认：static fold 的无 projected emission 路径消费了仅按 evidence 选择物化的公开 SourceMap。见 [编译/行为原流](evidence-selection/compile-oracle.json)、[耦合位置与后续验收](evidence-selection/analysis.md)。这是原 E2-dev 的诊断，没有增加独立 lineage 或矩阵分母。

例如 NoBody 可进入 pipeline，但不会产生 Body/raw CFG；表发布也可能是可靠 partial 前缀，不能把上述次数叫完整恢复次数。读取计费、真实读取和 IR 工作分别报告，不再用 `ir_items` 或 facade Body 次数替代底层事件。

WAR bulk 的每次守卫失败保存 250ms 观测前缀，见 [原始记录](resource-raw.json)。最后捕获峰值仅是终止前下界，不填成完整峰值，也不从对象数推断 RSS 的分配根因。守卫按 0.5s 轮询进程组 RSS，可能超过 1 GiB 才被终止；这不是 OS 硬内存上限。

| 故障 | 返回样本 | 缺 Prepared 样本 | 返回后活 worker 最大值 |
| --- | ---: | ---: | ---: |
{chr(10).join(fault_lines)}

token 在真实 Prepared 后并发置位，保存置位到返回以及实际 Body/IR 工作的前后快照边界；不用完成计数冒充新启动次数。elapsed 仅验证预过期路径，慢 sink 的缓冲容量单列。Refused 无 Prepared 合法，sink 拒绝后不强补事件。panic 缺 Prepared 是否复现：**{missing_prepared}**。这些是已测产品失败基线，未修复独立债务 E1/E2（与机制矩阵 E1/E2 问题族编号不同）。见 [独立失败路径](failure-path-results.json)。

## 后续阶段的输入

- S1：使用完整环境/身份和事实成本基线，保留当前有效成功正文，关闭已复现的 evidence/类族恢复耦合；独立 SourceTarget、需求去重与职责迁移尚未实现。
- S2：T1/T2 和真实类型/变量失败；必须新增循环 phi 同时复制等未覆盖域的受控对照。
- S3：E1/E2、数值边界与效果轨迹；可编译 fallback 仍按原 class 核对行为。
- S4：R1/R2、catch/循环组合与跨界变量；消除共同归属障碍，不能给 R1-dev-B 特判。
- S5：C1/C2、原目标家族隔离编译与开放闭包负例；整类来源和未知调用者边界继续保留。
- S6：query 零 Body/IR/AST、cache/慢 sink/token 和 E2 输入；不得掩盖单阶段完成范围。

handler 全域、完整数值 lowering、ECJ/legacy 的新增完整行为矩阵、动态加载/反射/未知 ABI 闭合没有在此证明。E1–E6 按 [设计](../../design.md) 的条件前置独立处理。S0 已完成，按用户限制在此交付结果，**暂不进入 S1，也不归档整个 change**。
'''
    (out/'README.md').write_text(text)
    write(out/'evidence-index.json',[pin(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='evidence-index.json'])
    print(json.dumps(dict(accepted=True,checks=checks,returns=len(samples),guarded=len(guards),focused_tests=len(test_rows)),indent=2))

if __name__=='__main__':
    import sys
    observed() if sys.argv[1:]==['observed'] else main()
