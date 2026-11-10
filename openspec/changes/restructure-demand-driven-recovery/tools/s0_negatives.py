#!/usr/bin/env python3
"""Freeze and replay existing refusal contracts, without running arbitrary target code."""
import importlib.util
import json
from pathlib import Path
import re
import subprocess

spec=importlib.util.spec_from_file_location('s0',Path(__file__).with_name('s0_baseline.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
OUT=b.CHANGE/'results/s0-resumed-v2/negatives'
BIN=Path('/private/tmp/jarde-s0-negative-bin-20261020')
CASES=[
 ('S2','T1','jarde_java','crates/jarde-java/src/build.rs','local_reference_write_compatibility_rejects_omitted_duplicate_and_pathless_accesses','incomplete/duplicate value-use constraints'),
 ('S2','T1','jarde_java','crates/jarde-java/src/build.rs','raw_alias_escape_rejects_ambiguous_or_incomplete_lvt_rows_atomically','ambiguous/incomplete variable lifetimes'),
 ('S2','T2','jarde_java','crates/jarde-java/src/build.rs','iterator_declaration_gate_uses_real_local_owner_and_refuses_incomplete_facts','generic declaration with incomplete owner/facts'),
 ('S3','E1','recover_postfix_old_value_snapshot','tests/recover_postfix_old_value_snapshot.rs','the_negatives_keep_their_refusals_on_both_legs','old-value/self-assignment and unsupported consumers'),
 ('S3','E1','jarde_java','crates/jarde-java/src/build.rs','static_prefix_rejects_real_non_one_subtract_and_old_value_consumers','effect absorption with extra old-value consumers'),
 ('S3','E2','jarde_java','crates/jarde-java/src/build.rs','exception_if_join_transfer_rejects_a_canonical_exception_edge','expression transfer across a protected exception edge'),
 ('S3','numeric','jarde_java','crates/jarde-java/src/build.rs','explicit_conversion_source_categories_accept_int_family_but_reject_boolean','numeric/boolean source category boundary'),
 ('S4','R1','jarde_java','crates/jarde-java/src/region.rs','range_end_transfer_rejects_competing_entries_and_exits','foreign normal/handler/Call entry or competing exit'),
 ('S4','R2','jarde_java','crates/jarde-java/src/region.rs','prefixed_loop_chain_certificate_rejects_extra_entries_exits_and_owners','overlap/extra ownership or transfer'),
 ('S5','C1','recover_static_generic_field_init_text','tests/recover_static_generic_field_init_text.rs','the_refusals_and_the_instance_field_control_survive','generic static-init refusals and instance-field control'),
 ('S5','C2','class_source','tests/class_source.rs','anonymous_inner_this_refuses_a_second_owner_allocation_and_keeps_physical_child','second allocation invalidates one anonymous projection'),
 ('S5','C2','class_source','tests/class_source.rs','anonymous_superclass_refuses_missing_parent_or_incomplete_child_body','missing parent or incomplete related method'),
 ('S5','C2','member_family_external_use','tests/member_family_external_use.rs','stopped_external_census_keeps_physical_family_and_refuses_projection','incomplete external-use closure is not zero uses'),
]

def main():
    assert not OUT.exists(), 'refuse overwriting a frozen run'
    OUT.mkdir(parents=True)
    lists={}
    for binary in sorted({c[2] for c in CASES}):
        meta,text=b.run([BIN/binary,'--list'],OUT/(binary+'-list'))
        assert meta['exit_code']==0
        lists[binary]=[s[:-6] for s in text.splitlines() if s.endswith(': test')]
    plan=[]
    for i,(stage,family,binary,source,function,boundary) in enumerate(CASES):
        names=[x for x in lists[binary] if x==function or x.endswith('::'+function)]
        assert len(names)==1,(function,names)
        src=b.ROOT/source
        matches=[i+1 for i,line in enumerate(src.read_text().splitlines()) if re.search(r'fn\s+'+re.escape(function)+r'\s*\(',line)]
        assert len(matches)==1,(function,matches)
        plan.append(dict(id=f'N{i+1:02}',stage=stage,family=family,binary=b.pin(BIN/binary),source=b.pin(src),line=matches[0],test=names[0],boundary=boundary,
            command=[str(BIN/binary),'--exact',names[0],'--nocapture','--test-threads=1'],
            scope='existing static fixture/proof-mutation refusal regression; not a full-class behavior oracle or a new holdout'))
    b.write(OUT/'plan.json',dict(engine_revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=b.ROOT,text=True).strip(),cases=plan))
    rows=[]
    for c in plan:
        meta,text=b.run(c['command'],OUT/c['id'])
        rows.append(dict(id=c['id'],stage=c['stage'],family=c['family'],exit_code=meta['exit_code'],one_pass='1 passed; 0 failed' in text,scope=c['scope']))
        b.write(OUT/'results.json',rows)
    print(json.dumps(rows,indent=2))

if __name__=='__main__':main()
