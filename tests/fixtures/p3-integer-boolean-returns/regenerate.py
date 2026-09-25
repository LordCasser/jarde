#!/usr/bin/env python3
"""Rebuild Java 8 source, descriptor-patched fixture and shared-stack variant."""
import hashlib, importlib.util, json, subprocess, sys
from pathlib import Path
here = Path(__file__).resolve().parent
repo = here.parents[2]
array_patcher = repo / 'tests/fixtures/p3-narrow-array-stores/patch_array_stores.py'
spec = importlib.util.spec_from_file_location('classpatch', array_patcher)
pm = importlib.util.module_from_spec(spec); spec.loader.exec_module(pm)
def code_digest(data, entries):
    chunks=[]
    for method in pm.find_methods(data, entries):
        for name_index, start, _ in method['attrs']:
            if pm.cp_utf8(entries, name_index) == 'Code':
                _, cur=pm.u2(data,start); _,cur=pm.u2(data,cur); length,cur=pm.u4(data,cur)
                chunks.append(bytes(data[cur:cur+length]))
    return hashlib.sha256(b''.join(chunks)).hexdigest()
def patch_class(src, dst, patches):
    data = bytearray(src.read_bytes()); entries, cp_end = pm.parse_cp(data)
    old_code = code_digest(data, entries)
    resolved = {}
    for name, (old, ret) in patches.items():
        desc = old[:old.rfind(')')+1] + ret
        data, entries, cp_end, idx = pm.cp_add_utf8(data, entries, cp_end, desc)
        resolved[name] = (old, desc, idx)
    seen=set()
    for method in pm.find_methods(data, entries):
        if method['name'] in resolved:
            old, new, idx = resolved[method['name']]
            if method['descriptor'] != old: raise ValueError((method['name'], method['descriptor'], old))
            pm.put_u2(data, method['info']+4, idx); seen.add(method['name'])
    if seen != set(resolved): raise ValueError(f'missing {set(resolved)-seen}')
    dst.parent.mkdir(parents=True, exist_ok=True); dst.write_bytes(data)
    out_entries,_=pm.parse_cp(data); code_hash=code_digest(data,out_entries)
    report={'source_sha256':hashlib.sha256(src.read_bytes()).hexdigest(),'output_sha256':hashlib.sha256(data).hexdigest(),'patches':{n:{'from':a,'to':b} for n,(a,b,_) in resolved.items()},'code_sha256_before':old_code,'code_sha256_after':code_hash}
    if old_code != code_hash: raise ValueError('descriptor patch changed method Code bytes')
    dst.with_suffix('.patch.json').write_text(json.dumps(report,indent=2)+'\n')

def main():
    if len(sys.argv)!=2: raise SystemExit('usage: regenerate.py OUTPUT_DIR')
    out=Path(sys.argv[1]); classes=out/'classes'; classes.mkdir(parents=True,exist_ok=True)
    subprocess.run(['javac','--release','8','-g:none','-d',str(classes),str(here/'IntegerBooleanReturns.java')],check=True)
    stage=out/'p3-integer-boolean-returns'
    main_dir=stage/'v8'; main_dir.mkdir(parents=True,exist_ok=True)
    patch_class(classes/'IntegerBooleanReturns.class',main_dir/'IntegerBooleanReturns.class',{
      'direct':('(I)I','Z'),'once':('(LIntegerBooleanReturns;I)I','Z'),
      'post':('()I','Z'),'pre':('()I','Z'),'sync':('(LIntegerBooleanReturns;I)I','Z'),
      'syncOn':('(Ljava/lang/Object;I)I','Z')})
    base=repo/'tests/fixtures/p3-narrow-integer-returns/actual-stack-join/v8/byte/ActualStackJoin.class'
    stack_dir=main_dir/'actual-stack-join'; stack_dir.mkdir(parents=True,exist_ok=True)
    patch_class(base,stack_dir/'ActualStackJoin.class',{'runByte':('(I)B','Z')})
    stack_runner=here/'actual-stack-join/ActualStackJoinRunner.java'
    subprocess.run(['javac','--release','8','-g:none','-cp',str(stack_dir),'-d',str(stack_dir),str(stack_runner)],check=True)
    subprocess.run(['java','-Xverify:all','-cp',str(stack_dir),'ActualStackJoinRunner'],check=True,stdout=(stack_dir/'expected.txt').open('w'))
    runner=here/'IntegerBooleanReturnsRunner.java'
    subprocess.run(['javac','--release','8','-g:none','-cp',str(main_dir),'-d',str(main_dir),str(runner)],check=True)
    subprocess.run(['java','-Xverify:all','-cp',str(main_dir),'IntegerBooleanReturnsRunner'],check=True,stdout=(main_dir/'expected.txt').open('w'))
if __name__=='__main__': main()
