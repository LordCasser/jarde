#!/usr/bin/env python3
"""Mutate compiled BooleanLocalRefusals.writeTwo's true store and emit physical class census."""
import argparse, hashlib, json
from pathlib import Path

class Reader:
    def __init__(self, data): self.data=data; self.p=0
    def u1(self): v=self.data[self.p]; self.p+=1; return v
    def u2(self): v=int.from_bytes(self.data[self.p:self.p+2], 'big'); self.p+=2; return v
    def u4(self): v=int.from_bytes(self.data[self.p:self.p+4], 'big'); self.p+=4; return v
    def take(self,n): v=self.data[self.p:self.p+n]; assert len(v)==n; self.p+=n; return v

def parse_class(data):
    r=Reader(data); assert r.u4()==0xCAFEBABE
    minor,major=r.u2(),r.u2(); count=r.u2(); cp=[None]*count; i=1
    while i<count:
        tag=r.u1()
        if tag==1: cp[i]=('utf8',r.take(r.u2()).decode('utf-8','replace'))
        elif tag in (3,4): cp[i]=('num',r.take(4))
        elif tag in (5,6): cp[i]=('wide',r.take(8)); i+=1
        elif tag in (7,8,16,19,20): cp[i]=('one',r.u2())
        elif tag in (9,10,11,12,17,18): cp[i]=('two',(r.u2(),r.u2()))
        elif tag==15: cp[i]=('handle',(r.u1(),r.u2()))
        else: raise ValueError(f'unsupported constant-pool tag {tag}')
        i+=1
    utf=lambda idx: cp[idx][1] if cp[idx] and cp[idx][0]=='utf8' else None
    access=r.u2(); this_class=r.u2(); super_class=r.u2(); internal=utf(cp[this_class][1])
    for _ in range(r.u2()): r.u2()
    def skip_members(n):
        for _ in range(n):
            r.u2(); r.u2(); r.u2()
            for _ in range(r.u2()): r.u2(); r.take(r.u4())
    field_count=r.u2(); field_rows=[]
    for _ in range(field_count):
        acc,name,desc=r.u2(),r.u2(),r.u2(); attrs=[]
        for _ in range(r.u2()): attrs.append((utf(r.u2()),r.take(r.u4())))
        field_rows.append({'name':utf(name),'descriptor':utf(desc),'access':acc})
    methods=[]
    for _ in range(r.u2()):
        acc,name,desc=r.u2(),r.u2(),r.u2(); attrs=[]; code=None
        for _ in range(r.u2()):
            attr_name=utf(r.u2()); n=r.u4(); start=r.p; info=r.take(n); attrs.append(attr_name)
            if attr_name=='Code':
                q=Reader(info); max_stack=q.u2(); max_locals=q.u2(); ncode=q.u4(); local_code_start=start+q.p; bytes_=q.take(ncode)
                exception_rows=[]
                for _ in range(q.u2()): exception_rows.append([q.u2(),q.u2(),q.u2(),q.u2()])
                lvt=[]
                for _ in range(q.u2()):
                    attr=utf(q.u2()); size=q.u4(); nested=Reader(q.take(size))
                    if attr=='LocalVariableTable':
                        for _ in range(nested.u2()):
                            begin,length,ni,di,slot=nested.u2(),nested.u2(),nested.u2(),nested.u2(),nested.u2()
                            lvt.append({'start':begin,'end':begin+length,'name':utf(ni),'descriptor':utf(di),'slot':slot})
                    else: nested.take(len(nested.data))
                assert q.p==len(info)
                code={'max_stack':max_stack,'max_locals':max_locals,'bytes':bytes_,'absolute_code_start':local_code_start,'exceptions':exception_rows,'locals':lvt}
        methods.append({'name':utf(name),'descriptor':utf(desc),'access':acc,'attributes':attrs,'code':code})
    for _ in range(r.u2()): r.u2(); r.take(r.u4())
    assert r.p==len(data)
    return {'minor':minor,'major':major,'internal_name':internal,'access':access,'fields':field_rows,'methods':methods}

def instructions(code):
    b=code; out=[]; pc=0
    while pc<len(b):
        start=pc; op=b[pc]; pc+=1; targets=[]
        if op in (0x10,0x12,0x15,0x16,0x17,0x18,0x19,0x36,0x37,0x38,0x39,0x3a,0xa9,0xbc): pc+=1
        elif op in (0x11,0x13,0x14,0x84) or 0x99<=op<=0xa8 or op in (0xc6,0xc7,0xc0,0xc1,0xb2,0xb3,0xb4,0xb5,0xb6,0xb7,0xb8,0xbb,0xbd):
            if op in (0x99,0x9a,0x9b,0x9c,0x9d,0x9e,0x9f,0xa0,0xa1,0xa2,0xa3,0xa4,0xa5,0xa6,0xa7,0xa8,0xc6,0xc7):
                off=int.from_bytes(b[pc:pc+2],'big',signed=True); targets=[start+off]
            pc+=2
        elif op in (0xb9,0xba): pc+=4
        elif op==0xc5: pc+=3
        elif op in (0xc8,0xc9):
            off=int.from_bytes(b[pc:pc+4],'big',signed=True); targets=[start+off]; pc+=4
        elif op==0xc4:
            sub=b[pc]; pc+=1
            if sub==0x84: pc+=4
            else: pc+=2
        elif op in (0xaa,0xab):
            while pc%4: pc+=1
            default=int.from_bytes(b[pc:pc+4],'big',signed=True); targets.append(start+default); pc+=4
            if op==0xaa:
                low=int.from_bytes(b[pc:pc+4],'big',signed=True); high=int.from_bytes(b[pc+4:pc+8],'big',signed=True); pc+=8
                for _ in range(high-low+1): targets.append(start+int.from_bytes(b[pc:pc+4],'big',signed=True)); pc+=4
            else:
                pairs=int.from_bytes(b[pc:pc+4],'big',signed=True); pc+=4
                for _ in range(pairs): pc+=4; targets.append(start+int.from_bytes(b[pc:pc+4],'big',signed=True)); pc+=4
        if pc>len(b): raise ValueError(f'truncated opcode {op:02x} at {start}')
        out.append({'bci':start,'opcode':op,'length':pc-start,'targets':targets})
    return out

def local_store(ins,code):
    op=ins['opcode']; pc=ins['bci']
    if op==0x36: return code[pc+1]
    if 0x3b<=op<=0x3e: return op-0x3b
    return None

def pin(path):
    b=path.read_bytes(); return {'path':str(path),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}

def transform(src,dst):
    raw=src.read_bytes(); parsed=parse_class(raw)
    assert parsed['major']==52, (src,parsed['major'])
    assert parsed['internal_name']=='cf09/literalboolean/BooleanLocalRefusals'
    method=[m for m in parsed['methods'] if m['name']=='writeTwo' and m['descriptor']=='(Z)Z']
    assert len(method)==1 and method[0]['code'] is not None
    code=method[0]['code']; ins=instructions(code['bytes'])
    found=[x for x in code['locals'] if x['name']=='found' and x['descriptor']=='Z']
    assert len(found)==1, f'{src}: debug LVT must identify one found:Z local, got {found}'
    local=found[0]; slot=local['slot']
    stores=[]
    for i,x in enumerate(ins):
        if local_store(x,code['bytes'])==slot: stores.append((i,x))
    assert len(stores)==2, f'{src}: expected exactly two writes to found slot {slot} across the full method, got {stores}'
    # javac starts this local's LVT after the initial store. Require that store to end exactly
    # at the LVT start, then require the second write to fall within the one found scope.
    init=[(i,x) for i,x in stores if i and ins[i-1]['opcode']==0x03 and ins[i-1]['bci']+ins[i-1]['length']==x['bci']]
    true=[(i,x) for i,x in stores if i and ins[i-1]['opcode']==0x04 and ins[i-1]['bci']+ins[i-1]['length']==x['bci']]
    assert len(init)==1 and init[0][1]['bci']+init[0][1]['length']==local['start'], f'{src}: expected one direct iconst_0 initializer immediately before LVT start {local["start"]}, got {init}'
    assert len(true)==1 and local['start']<=true[0][1]['bci']<local['end'], f'{src}: expected one direct iconst_1 store within found LVT scope, got {true}'
    literal_bci,store_bci=true[0][1]['bci']-ins[true[0][0]-1]['length'],true[0][1]['bci']
    changed=bytearray(raw); changed[code['absolute_code_start']+literal_bci]=0x05
    dst.parent.mkdir(parents=True,exist_ok=True); dst.write_bytes(changed)
    reread=parse_class(bytes(changed)); updated=[m for m in reread['methods'] if m['name']=='writeTwo' and m['descriptor']=='(Z)Z'][0]
    assert updated['code']['bytes'][literal_bci]==0x05
    diffs=[i for i,(a,b) in enumerate(zip(raw,changed)) if a!=b]
    assert diffs==[code['absolute_code_start']+literal_bci]
    return {'compiled_input':pin(src),'mutated_output':pin(dst),'target_method':'writeTwo(Z)Z','found_slot':slot,'literal_bci':literal_bci,'store_bci':store_bci,'method_inventory':[{k:m[k] for k in ('name','descriptor','access')} for m in parsed['methods']],'field_inventory':parsed['fields'],'code_census':{m['name']+m['descriptor']:{'max_stack':m['code']['max_stack'],'max_locals':m['code']['max_locals'],'code_length':len(m['code']['bytes']),'code_hex':m['code']['bytes'].hex(),'exception_rows':m['code']['exceptions'],'instructions':instructions(m['code']['bytes'])} for m in parsed['methods'] if m['code']}}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--javac8-class',type=Path,required=True); ap.add_argument('--javac23-class',type=Path,required=True); ap.add_argument('--out',type=Path,required=True); a=ap.parse_args()
    records={'schema':'cf09-literal-boolean-controls-generated-class-census-v1','status':'caller-compiled-classfiles-mutated-no-compiler-invoked-by-this-tool','outputs':{}}
    for label,src in [('jdk8',a.javac8_class),('jdk23',a.javac23_class)]:
        dst=a.out/label/'BooleanLocalRefusals.class'; records['outputs'][label]=transform(src,dst)
    a.out.mkdir(parents=True,exist_ok=True); (a.out/'census.json').write_text(json.dumps(records,indent=2)+'\n')
    print(json.dumps(records,indent=2))
if __name__=='__main__': main()
