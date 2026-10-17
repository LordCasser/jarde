from pathlib import Path
import json,hashlib
D=Path(__file__).resolve().parent
W=Path('/home/lordcasser/.codex/worktrees/core-integrity/jarde')
main=W/'openspec/specs/java8-recovery/spec.md';delta=W/'openspec/changes/recover-loop-transfer-finally-handler/specs/java8-recovery/spec.md';analysis=W/'openspec/specs/analysis-contracts/spec.md';tasks=W/'openspec/changes/recover-loop-transfer-finally-handler/tasks.md'
old=main.read_bytes();raw=delta.read_bytes();ac=analysis.read_bytes();task=tasks.read_text();assert task.count('- [x] ')==6 and '- [ ] ' not in task
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(old)=='6ed57651c0fa7f1591997c8c51edc0454a651d89b394c588de42771f3f2dd54c' and old.count(b'### Requirement:')==23 and old.count(b'#### Scenario:')==156
assert sha(ac)=='668af66f52a79c199abc188834995c5d197453ac9d5659eadd6a1e688c211684'
assert raw.count(b'### Requirement:')==1 and raw.count(b'#### Scenario:')==8 and b'## ADDED Requirements' in raw
addition=b'\n'+raw[raw.index(b'### Requirement:'):];title=addition.splitlines()[1];assert title not in old
new=old+addition;assert new.startswith(old) and new.count(b'### Requirement:')==24 and new.count(b'#### Scenario:')==164
for name,data in [('main-before.md',old),('delta.md',raw),('addition.md',addition),('main-proposed.md',new),('analysis-contracts-before.md',ac)]:
 p=D/name;assert not p.exists();p.write_bytes(data)
main.write_bytes(new);assert main.read_bytes()==new and analysis.read_bytes()==ac
result=dict(status='applied_one_added_requirement_eight_scenarios_old_bytes_prefix_preserved',old_sha256=sha(old),new_sha256=sha(new),old_requirements=23,new_requirements=24,old_scenarios=156,new_scenarios=164,analysis_contracts_sha256=sha(ac),task_count=6,script_sha256=sha(Path(__file__).read_bytes()))
(D/'sync-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
