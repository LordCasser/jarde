#!/usr/bin/env python3
"""Read-only supplement for the S0 method runtime-binding denominator."""
import gzip, json, pathlib, collections, re
ROOT=pathlib.Path(__file__).resolve().parents[4]
BASE=ROOT/'openspec/changes/restructure-demand-driven-recovery/results/s0-baseline-v1'
OUT=ROOT/'openspec/changes/restructure-demand-driven-recovery/results/s0-resumed-v2/census'
ARTS={'bcprov':pathlib.Path('/Users/lordcasser/workspace/vulnerability/vulhub/weblogic/weak_password/decrypt/lib/bcprov-jdk15on-152.jar'),
      's2-009':pathlib.Path('/Users/lordcasser/workspace/vulnerability/vulhub/struts2/s2-009/S2-009.war')}

def canon(x): return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False)
def classkey(defn): return canon({k:defn.get(k) for k in ('location','class_bytes','variant')})
def name_bytes(item): return item.get('declaration',{}).get('this_class',{}).get('raw',[])
def read_name(r):
 d=r.get('definition',{})
 return classkey(d)
def diag_all(value,out=None):
 if out is None: out=[]
 if isinstance(value,dict):
  if isinstance(value.get('code'),str) and isinstance(value.get('message'),str): out.append(value)
  for k,v in value.items():
   if k not in ('text','source_map','usage','artifact','environment_identity'): diag_all(v,out)
 elif isinstance(value,list):
  for v in value: diag_all(v,out)
 return out

report={'method_binding_denominator':{},'unbound_class_groups':[],'source_locations':{
 'bulk_prepared_entry':'src/bulk.rs:2066-2086',
 'prepared_method_binding_and_no_body_order':'crates/jarde-jvm/src/engine.rs:1452-1506',
 'direct_method_binding_contract':'crates/jarde-jvm/src/engine.rs:1091-1145',
 'read_and_bind_definition':'crates/jarde-jvm/src/providers.rs:813-897',
 'unbound_message_selected_definition':'crates/jarde-jvm/src/providers.rs:2224-2252',
 'report_reads_contract':'crates/jarde-jvm/src/ir.rs:299-314',
}}
all_unbound=[]
for label in ARTS:
 listing=json.loads(gzip.open(BASE/f'inventory/{label}-classes.stdout.gz','rt').read())
 by_def={classkey(item['definition']):item for item in listing.get('items',[])}
 by_internal=collections.defaultdict(list)
 for item in listing.get('items',[]):
  rb=name_bytes(item)
  by_internal[bytes(rb)].append(item)
 cross=collections.Counter(); read_counts=collections.Counter(); loader_counts=collections.Counter(); read_exact_counts=collections.Counter(); groups={}
 with gzip.open(BASE/f'bulk/{label}/records.jsonl.gz','rt') as stream:
  for line in stream:
   event=json.loads(line)
   if event.get('kind')!='method': continue
   delivery=event.get('delivery',{}); analysis=delivery.get('analysis',{})
   owner=event.get('method',{}).get('owner',{})
   own_key=classkey(owner)
   own_item=by_def.get(own_key)
   raw_name=bytes(name_bytes(own_item)) if own_item else None
   parsed_code=None
   # Previous independent census joined exact member ordinal to the independent classfile shell.
   meth=event.get('method',{})
   # use independent row identity rather than any exporter counter
   # match rows in the saved per-method census stream by physical class + method ordinal only when needed below
   # Code status is recovered from the immutable derived private census file.
   stage={x.get('stage'):x.get('state',{}) for x in analysis.get('stages',[])}
   rf=stage.get('raw_facts',{})
   rfkind=rf.get('kind','missing')
   code_present=None
   # Stable key for direct join from precomputed census.
   ident=(event.get('class_ordinal'),event.get('member_ordinal'))
   reads=analysis.get('reads',[])
   loader=analysis.get('loader')
   codes={d.get('code') for d in diag_all(delivery) if isinstance(d,dict)}
   unbound='resolution_definition_unbound' in codes
   body=(analysis.get('body') or {}).get('kind')
   bound=(rfkind=='completed' and body=='present') or (delivery.get('state')=='no_body' and body=='declared_without_body' and len(reads)>0 and not unbound)
   # Every read is retained as full physical definition identity, loader and demand reason.
   matched_own_read=any(read_name(x)==own_key and x.get('reason')=='driver_method_body' for x in reads)
   read_counts['method_records']+=1
   read_counts['has_driver_method_body_read']+=bool(any(x.get('reason')=='driver_method_body' for x in reads))
   read_counts['claimed_definition_in_reads']+=bool(matched_own_read)
   loader_counts[loader or 'missing']+=1
   if bound: bind_state='bound'
   elif unbound: bind_state='explicit_unbound'
   else: bind_state='unresolved_or_other_failure'
   # Include code status via exact member ordinal from previously verified per-method rows (small in-memory map lazily loaded).
   cross[(bind_state,rfkind,body,delivery.get('state'))]+=1
   if unbound:
    diags=[d for d in diag_all(delivery) if d.get('code')=='resolution_definition_unbound']
    msg=diags[0].get('message','') if diags else ''
    selected=bool(re.search(r'it selects the definition ',msg))
    missing='no position of its order holds that name at all' in msg
    ambiguous='its first position holds indistinguishable candidates' in msg
    same_name=by_internal.get(raw_name or b'',[])
    same_name_other=[]
    for candidate in same_name:
     if classkey(candidate['definition']) != own_key:
      same_name_other.append(candidate)
    # Prove shadow only if selected identity in reads maps to same raw class name and differs from claim.
    selected_reads=[]
    for read in reads:
     rkey=read_name(read)
     if rkey==own_key: continue
     selected_item=by_def.get(rkey)
     if selected_item and bytes(name_bytes(selected_item))==raw_name:
      selected_reads.append(dict(definition=read.get('definition'),reason=read.get('reason'),loader=read.get('loader'),
                                 entry_name=selected_item.get('definition',{}).get('location',{}).get('entry',{}).get('raw_name'),
                                 internal_name_raw=name_bytes(selected_item)))
    group_id=canon({'artifact':label,'claimed':owner,'internal_name_raw':list(raw_name or b'')})
    group=groups.setdefault(group_id,dict(artifact=label,class_ordinal=event.get('class_ordinal'),
      class_internal_name_raw=list(raw_name or b''),class_internal_name=(raw_name or b'').decode('utf-8',errors='replace'),
      claimed_definition=owner,method_count=0,methods=[],loader=loader,unbound_message=msg,
      same_name_physical_definition_count=len(same_name),same_name_other_origins=[x.get('definition') for x in same_name_other],
      selected_same_name_other_definitions=selected_reads,
      shadow_status=('confirmed_selected_other_definition' if selected and selected_reads else
                     'selected_other_label_but_read_not_matched' if selected else
                     'no_candidate_in_order' if missing else
                     'ambiguous_first_position' if ambiguous else 'cause_not_resolved')))
    group['method_count']+=1
    group['methods'].append(dict(member_ordinal=event.get('member_ordinal'),name_raw=meth.get('name'),descriptor_raw=meth.get('descriptor'),
      independent_code_present=None,raw_facts_state=rf,body=analysis.get('body'),loader=loader,
      reads=reads,unbound_message=msg))
    all_unbound.append((label,event.get('class_ordinal'),event.get('member_ordinal'),own_key))

 # Fill independent Code status counts and method details from saved full census, matching exact class ordinal/member ordinal.
 code_map={}
 with gzip.open(OUT/f'{label}-methods.jsonl.gz','rt') as census:
  for line in census:
   row=json.loads(line); code_map[(row['class_ordinal'],row['member_ordinal'])]=row.get('independent_code_present')
 with gzip.open(BASE/f'bulk/{label}/records.jsonl.gz','rt') as stream:
  code_cross=collections.Counter(); reads_shape=collections.Counter()
  for line in stream:
   event=json.loads(line)
   if event.get('kind')!='method': continue
   a=event.get('delivery',{}).get('analysis',{}); d=event.get('delivery',{})
   code=code_map.get((event.get('class_ordinal'),event.get('member_ordinal')))
   st={x.get('stage'):x.get('state',{}) for x in a.get('stages',[])}.get('raw_facts',{})
   unbound=any(x.get('code')=='resolution_definition_unbound' for x in diag_all(d))
   body=(a.get('body') or {}).get('kind'); reads=a.get('reads',[])
   is_bound=(st.get('kind')=='completed' and body=='present') or (d.get('state')=='no_body' and body=='declared_without_body' and bool(reads) and not unbound)
   cstate='bound' if is_bound else 'explicit_unbound' if unbound else 'other'
   code_cross[(('Code' if code else 'NoCode' if code is False else 'CodeUnknown'),cstate)]+=1
   if d.get('state')=='no_body':
    reads_shape[(a.get('loader'),len(reads),tuple(x.get('reason') for x in reads),all(read_name(x)==classkey(event['method']['owner']) for x in reads))]+=1
 report['method_binding_denominator'][label]=dict(
  total_method_records=sum(code_cross.values()),
  cross_tab=[dict(code_status=k[0],binding_status=k[1],methods=n) for k,n in sorted(code_cross.items())],
  payload_contract='A no_body result is returned only after prepared binding succeeds and member body is classified; a failed bind exits before it. Bound means raw_facts completed/body present, or no_body declared_without_body with driver_method_body read and no unbound diagnostic.',
  no_body_read_shape=[dict(loader=k[0],read_count=k[1],reasons=list(k[2]),all_reads_match_claimed_definition=k[3],methods=n) for k,n in reads_shape.items()])
 report['unbound_class_groups'].extend(groups.values())
(OUT/'runtime-binding-supplement.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
print('output',OUT/'runtime-binding-supplement.json')
print(json.dumps(report['method_binding_denominator'],indent=2,ensure_ascii=False))
print('unbound class groups',len(report['unbound_class_groups']))
print('shadow statuses',dict(collections.Counter(g['shadow_status'] for g in report['unbound_class_groups'])))
print('unbound methods',len(all_unbound))
