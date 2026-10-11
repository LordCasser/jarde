#!/usr/bin/env python3
import hashlib, json, pathlib, re, struct, sys
OUT=pathlib.Path('/private/tmp/jarde-computed-for-controls-root-v1')
D=json.loads((OUT/'execution.json').read_text())
def ck(v,m):
    if not v: raise AssertionError(m)
def sha(p): return hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
def u2(b,p): return struct.unpack_from('>H',b,p)[0]
def u4(b,p): return struct.unpack_from('>I',b,p)[0]
def parse_class(b):
    ck(b[:4]==b'\xca\xfe\xba\xbe','class magic')
    n=u2(b,8); cp=[None]*n; p=10; i=1
    while i<n:
        tag=b[p]; p+=1
        if tag==1:
            size=u2(b,p); p+=2; cp[i]=b[p:p+size].decode('ascii'); p+=size
        elif tag in (3,4): p+=4
        elif tag in (5,6): p+=8; i+=1
        elif tag==7: cp[i]=('class',u2(b,p)); p+=2
        elif tag==12: cp[i]=('nat',u2(b,p),u2(b,p+2)); p+=4
        elif tag in (10,11): cp[i]=('ref',u2(b,p),u2(b,p+2)); p+=4
        elif tag in (9,17,18): p+=4
        elif tag in (8,16,19,20): p+=2
        elif tag==15: p+=3
        else: raise AssertionError('unknown cp tag '+str(tag))
        i+=1
    cp_end=p; p+=6; ni=u2(b,p); p+=2+2*ni; methods={}
    nf=u2(b,p); p+=2
    def skip_attrs(p,count):
        for _ in range(count): p+=6+u4(b,p+2)
        return p
    for _ in range(nf):
        p+=6; na=u2(b,p); p+=2; p=skip_attrs(p,na)
    nm=u2(b,p); p+=2
    for _ in range(nm):
        _,name_i,desc_i,na=struct.unpack_from('>HHHH',b,p); p+=8
        name,desc=cp[name_i],cp[desc_i]
        for _ in range(na):
            ai=u2(b,p); length=u4(b,p+2); body=p+6; aname=cp[ai]
            if aname=='Code':
                clen=u4(b,body+4); cstart=body+8
                methods[(name,desc)]={'code_start':cstart,'code_len':clen,'attr_start':p,'attr_end':body+length,'code':b[cstart:cstart+clen]}
            p=body+length
    return cp,cp_end,methods

def method_summary(report):
    out=[]
    for m in report['methods']:
        name=bytes(m['item']['identity']['name']).decode('ascii')
        r=m['outcome'].get('report',{})
        quality=r.get('quality')
        out.append({'name':name,'quality':quality,'has_for':'for (' in m['text']})
    return out

def main():
    ck(D['status']=='passed_original_and_pending_legality_and_observation_only','status boundary')
    ck(len(D['commands'])==20,'20 command records')
    ck(len(D['profiles'])==8 and len(D['runtime_pairs'])==2,'8 profiles / 2 runtime pairs')
    pins={x['path']:x['sha256'] for x in D['input_pins']}
    ck(all(sha(x['path'])==x['sha256'] for x in D['input_pins']),'input/JDK/CLI pins')
    ck(sha('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2')==pins['/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2'],'frozen CLI pin')
    ck(sha('/private/tmp/jarde-computed-for-controls-root-v1.py')==D['runner_sha256'],'execution runner script pin')
    labels=set()
    for c in D['commands']:
        label=c['label']; ck(label not in labels,'duplicate command label '+label); labels.add(label)
        ck(c['exit_code']==0,'exit code '+label)
        for stream in ('stdout','stderr'):
            raw=OUT/(label+'.'+stream+'.raw')
            ck(sha(raw)==c[stream+'_sha256'],'raw hash '+label+'/'+stream)
    ck(len(labels)==20,'all raw command rows')
    # Validate command families, exact JDK/CLI executables and Java verifier flags.
    for c in D['commands']:
        label=c['label']; a=c['argv']; prof=label.split('-',1)[0]
        if prof in ('javac8','javac23'):
            leg=prof
            tool=('javac' if label.endswith('-compile') else 'java' if label.endswith('-runtime') else 'javap' if label.endswith('-javap') else None)
            if tool: ck(pathlib.Path(a[0]).name==tool,'tool argv '+label)
            if label.endswith('-runtime'): ck('-Xverify:all' in a,'-Xverify:all '+label)
            if label.endswith('-compile'):
                ck('ComputedForControls.java' in a[-2] and 'ComputedForControlsRunner.java' in a[-1],'compile inputs '+label)
                if leg=='javac8': ck(a[1:6]==['-source','8','-target','8','-g:none'],'javac8 flags')
                else: ck('--release' in a and a[a.index('--release')+1]=='8','javac23 release')
        if label.endswith('-render'):
            ck(a[0]=='/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2','renderer CLI')
            ck(a[1:2]==['class-source'] and a[a.index('--class')+1]=='ComputedForControls','render request')
            ck('--format' in a and a[a.index('--format')+1]=='json','render format')
    # Bind every recorded argv to its exact source/class/profile path and mode.
    bylabel={c['label']:c for c in D['commands']}
    source='/private/tmp/jarde-computed-for-controls-luna-v1/ComputedForControls.java'
    runner='/private/tmp/jarde-computed-for-controls-luna-v1/ComputedForControlsRunner.java'
    patcher='/private/tmp/jarde-computed-for-controls-luna-v1/patch_pending_stack.py'
    for leg in ('javac8','javac23'):
        javac=next(x['path'] for x in D['input_pins'] if x['path'].endswith('/bin/javac') and (('corretto' in x['path'])==(leg=='javac8')))
        java=next(x['path'] for x in D['input_pins'] if x['path'].endswith('/bin/java') and (('corretto' in x['path'])==(leg=='javac8')))
        javap=next(x['path'] for x in D['input_pins'] if x['path'].endswith('/bin/javap') and (('corretto' in x['path'])==(leg=='javac8')))
        original=str(OUT/leg/'original'); pending=str(OUT/leg/'pending')
        compile_args=([javac,'-source','8','-target','8','-g:none','-proc:none','-d',original,source,runner] if leg=='javac8' else [javac,'--release','8','-g:none','-proc:none','-d',original,source,runner])
        ck(bylabel[leg+'-compile']['argv']==compile_args,'exact compile argv '+leg)
        ck(bylabel[leg+'-patch']['argv']==['python3',patcher,original+'/ComputedForControls.class',pending+'/ComputedForControls.class'],'exact patch argv '+leg)
        for side,cp in [('original',original),('pending',pending)]:
            ck(bylabel[f'{leg}-{side}-javap']['argv']==[javap,'-v','-c','-p','-classpath',cp,'ComputedForControls'],'exact javap argv '+leg+'/'+side)
            ck(bylabel[f'{leg}-{side}-runtime']['argv']==[java,'-Xverify:all','-cp',cp,'ComputedForControlsRunner'],'exact runtime argv '+leg+'/'+side)
            for evidence in ('default','all'):
                classfile=cp+'/ComputedForControls.class'
                args=['/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2','class-source','--input',classfile,'--class','ComputedForControls','--policy','single-class','--release','8','--format','json']
                if evidence=='all': args += ['--evidence','all']
                ck(bylabel[f'{leg}-{side}-{evidence}-render']['argv']==args,'exact renderer argv '+leg+'/'+side+'/'+evidence)
    source_pins={x['path']:x['sha256'] for x in D['input_pins']}
    ck(sha(source)==source_pins[source] and sha(runner)==source_pins[runner],'source and runner pins')
    # Original and pending are byte-identical outside the two target Code arrays.
    expected={
      'javac8':('4c253eb463a8e5bf105c2814e080cf31eb31235dca1a8b98ed7c8d2f3e471dff','f28859e6823aa643cedeb7ead216a7ee5debcd9ba35d241439c3b60d7435ee81','04643eb80007','0464b800073e','04643db80007','0464b800073d'),
      'javac23':('1baf82bd5b2e9c2e79777e6393e49c3dca433344c223ad30b94192a28295ccad','325dee2da3e623b3bac2bef1219b8dfca95a166d9f58315f65003859d277f14e','04643eb8001b','0464b8001b3e','04643db8001b','0464b8001b3d')}
    for leg,vals in expected.items():
        oldh,newh,bf,af,bs,ass=vals
        op=OUT/leg/'original'/'ComputedForControls.class'; np=OUT/leg/'pending'/'ComputedForControls.class'
        ob,nb=op.read_bytes(),np.read_bytes(); ck(sha(op)==oldh and sha(np)==newh,'class SHA '+leg)
        oc,oe,om=parse_class(ob); nc,ne,nm=parse_class(nb)
        ck(oe==ne and ob[:oe]==nb[:ne],'constant pool bytes changed '+leg)
        ck(len(ob)==len(nb),'class length changed '+leg)
        keys=[('find','([III)I'),('shared','(I)I')]
        ranges=[]
        for key in keys:
            x,y=om[key],nm[key]; ck(x['code_len']==y['code_len'],'Code length '+leg+'/'+key[0])
            before,after=(bytes.fromhex(bf),bytes.fromhex(af)) if key[0]=='find' else (bytes.fromhex(bs),bytes.fromhex(ass))
            bci=3 if key[0]=='find' else 5
            ck(x['code'][bci:bci+len(before)]==before and y['code'][bci:bci+len(after)]==after,'exact patch bytes '+leg+'/'+key[0])
            ck(x['code'][:bci]+after+x['code'][bci+len(before):]==y['code'],'unexpected Code edit '+leg+'/'+key[0])
            ranges.append((x['code_start']+bci,x['code_start']+bci+len(before)))
        masked=bytearray(len(ob))
        for lo,hi in ranges: masked[lo:hi]=b'\x01'*(hi-lo)
        ck(all(masked[i] or ob[i]==nb[i] for i in range(len(ob))),'non-target class bytes changed '+leg)
        for key in om:
            if key not in keys: ck(om[key]['code']==nm[key]['code'],'other method Code changed '+leg+'/'+key[0])
        runner=OUT/leg/'original'/'ComputedForControlsRunner.class'; copied=OUT/leg/'pending'/'ComputedForControlsRunner.class'
        ck(sha(runner)==sha(copied),'runner not copied byte-for-byte '+leg)
        ck(runner.is_file(),'runner class exists '+leg)
    # Raw javap proves BCI and stack choreography, not merely patch manifest claims.
    for leg in expected:
        for side in ('original','pending'):
            text=(OUT/f'{leg}-{side}-javap.stdout.raw').read_text()
            for method,store_bci,call_bci in [('find',5,6),('shared',7,8)]:
                start=text.index('static int '+method+'('); end=text.find('\n  static ',start+1); block=text[start:end if end>=0 else len(text)]
                if side=='original' and method=='find':
                    ck(re.search(r'\n\s*3: iconst_1\n\s*4: isub\n\s*5: istore_\d+\n\s*6: invokestatic\s+#\d+\s+// Method g:\(\)V',block) is not None,'original javap find BCI '+leg)
                elif side=='original' and method=='shared':
                    ck(re.search(r'\n\s*5: iconst_1\n\s*6: isub\n\s*7: istore_\d+\n\s*8: invokestatic\s+#\d+\s+// Method g:\(\)V',block) is not None,'original javap shared BCI '+leg)
                elif side=='pending' and method=='find':
                    ck(re.search(r'\n\s*3: iconst_1\n\s*4: isub\n\s*5: invokestatic\s+#\d+\s+// Method g:\(\)V\n\s*8: istore_\d+',block) is not None,'pending javap find BCI '+leg)
                else:
                    ck(re.search(r'\n\s*5: iconst_1\n\s*6: isub\n\s*7: invokestatic\s+#\d+\s+// Method g:\(\)V\n\s*10: istore_\d+',block) is not None,'pending javap shared BCI '+leg)
    expected_stdout=('hit-after-g-mutates-end:2:12:2\nmiss:-1:12:2\nf-throws-before-g:-2:1:6\ng-throws-after-f:-2:12:2\nshared-f-value:599:12:2\npost-loop-read:-1:1:6\npost-loop-write:0:1:6\nnested-scope-enter:-1:1:6\nnested-scope-skip:-7:0:6\n').encode()
    for pair in D['runtime_pairs']:
        leg=pair['profile']; ck(pair['equal'],'runtime equality '+leg)
        ck(pair['stdout'].encode()==expected_stdout,'nine-row summary '+leg)
        ck((OUT/leg/'expected-runtime.stdout.raw').read_bytes()==expected_stdout,'nine-row expected raw '+leg)
        for side in ('original','pending'):
            raw=(OUT/f'{leg}-{side}-runtime.stdout.raw').read_bytes()
            ck(raw==expected_stdout,'nine-row runtime raw '+leg+'/'+side)
            ck((OUT/f'{leg}-{side}-runtime.stderr.raw').read_bytes()==b'','runtime stderr '+leg+'/'+side)
    # Each profile report and emitted source are pinned; default/all text and maps must agree.
    ck(len(D['profiles'])==8,'profile rows')
    by={(p['profile'],p['side'],p['evidence']):p for p in D['profiles']}
    wanted_quality={'<init>':'structured','f':'structured','g':'structured','find':'fallback','shared':'fallback','observedAfter':'structured','externalWrite':'structured','scoped':'fallback','<clinit>':'structured'}
    for (leg,side,evidence),p in by.items():
        cls=OUT/leg/side/'ComputedForControls.class'; ck(sha(cls)==p['input_class_sha256'],'profile input hash '+str((leg,side,evidence)))
        raw=OUT/f'{leg}-{side}-{evidence}-render.stdout.raw'; report=json.loads(raw.read_text())
        emitted=(OUT/leg/f'{side}-{evidence}.java')
        ck(sha(emitted)==p['source_sha256'],'emitted source hash '+str((leg,side,evidence)))
        ck(emitted.read_text()==report['text'],'emitted source text '+str((leg,side,evidence)))
        rows=method_summary(report); ck([(r['name'],r['quality'],r['has_for']) for r in rows]==[(m['name'],m['quality'],m['has_for']) for m in p['methods']],'profile summary mismatch')
        ck({r['name']:r['quality'] for r in rows}==wanted_quality,'method quality '+str((leg,side,evidence)))
        ck(not any(r['has_for'] for r in rows),'unexpected for '+str((leg,side,evidence)))
    for leg in ('javac8','javac23'):
      for side in ('original','pending'):
        d=json.loads((OUT/f'{leg}-{side}-default-render.stdout.raw').read_text())
        a=json.loads((OUT/f'{leg}-{side}-all-render.stdout.raw').read_text())
        maps=lambda x:[(bytes(m['item']['identity']['name']).decode(),m['outcome'].get('report',{}).get('source_map')) for m in x['methods']]
        ck(d['text']==a['text'],'default/all text drift '+leg+'/'+side)
        ck(maps(d)==maps(a),'default/all source-map drift '+leg+'/'+side)
    print('PASS: 20 argv/raw records; input/CLI/JDK pins; two exact equal-length Code edits only; StackMap/CP and runner unchanged; javap BCI; 9 raw output rows × two JDKs; 8 report/source profiles and default/all text/maps')
if __name__=='__main__':
    try: main()
    except Exception as exc:
        print('FAIL:',exc,file=sys.stderr); raise
