from pathlib import Path
import hashlib,json,re
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-meet-char-root-v2');sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
d=json.loads((out/'execution.json').read_text());assert d['status']=='passed' and d['cli_sha256']=='e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403' and d['fixture_sha256']=='0e3b9f78d79e49e345ccb163a01eeaa0d21f9ed7ca0b340c24d7901cd4b1c04c'
assert sha('/private/tmp/jarde-proved-local-source-types-cli-v1')==d['cli_sha256'] and sha(root/'tests/fixtures/p3-meeting/v8/Meet.class')==d['fixture_sha256']
assert len(d['commands'])==19 and len(d['legs'])==8 and {(r['jdk'],r['profile']) for r in d['legs']}=={(j,p) for j in ('javac8','javac23') for p in ('original','default','all','jadx')}
for row in d['commands']:
 assert row['exit_code']==0
 for k in ('stdout','stderr'):
  f=Path(row[k]['path']);assert sha(f)==row[k]['sha256'] and f.stat().st_size==row[k]['bytes']
reports=[json.loads((out/f'{i}.stdout.raw').read_text()) for i in (0,1)];names={'<init>','at','viaStore','pass','fieldArg','viaStoreLong','trunc','grade','stat','viaRef','unchecked','pop2Control'}
for report in reports:
 assert len(report['methods'])==12 and {m['item']['name']['escaped'] for m in report['methods']}==names
 assert [m['item']['index'] for m in report['methods']]==list(range(12))
 assert all(m['item']['identity']['owner']['class_bytes']['digest']=='6aaf9411db348c862e3c3f32c76ed9fc2bf1e0729d5aea5b246bfa79758f1716' for m in report['methods'])
assert reports[0]['text']==reports[1]['text'] and reports[0]['methods']==reports[1]['methods']
runner='public class Runner { public static void main(String[] args) { long sum=0; for(int i=0;i<=65535;i++) { int value=Meet.viaStore((char)i); if(value!=i) throw new AssertionError("char="+i+", result="+value); sum+=value; } System.out.println("chars=65536,sum="+sum); } }\n'
for leg in d['legs']:
 j,p=leg['jdk'],leg['profile'];work=out/j/p;source=work/'Meet.java';rp=work/'Runner.java'
 assert sha(source)==leg['source_sha256'] and sha(rp)==leg['runner_sha256'];assert rp.read_text()==('package defpackage;\n' if p=='jadx' else '')+runner
 expected=(root/'tests/fixtures/p3-meeting/Meet.java').read_bytes() if p=='original' else (out/'jadx-output/sources/defpackage/Meet.java').read_bytes() if p=='jadx' else reports[0]['text'].encode()
 assert source.read_bytes()==expected
 runtime=leg['runtime'];assert runtime in d['commands'];assert Path(runtime['stdout']['path']).read_bytes()==b'chars=65536,sum=2147450880\n' and Path(runtime['stderr']['path']).read_bytes()==b''
 assert runtime['argv'][-4:]==['-Xverify:all','-cp',str(work/'classes'),'defpackage.Runner' if p=='jadx' else 'Runner']
 compile_rows=[r for r in d['commands'] if r['argv'][-2:]==[str(source),str(rp)]];assert len(compile_rows)==1
 assert (work/'classes'/('defpackage' if p=='jadx' else '')/'Meet.class').is_file()
result={'schema':'meet-char-int-whole-class-acceptance-root-v1','status':'accepted','execution_sha256':sha(out/'execution.json'),'commands':19,'raw_streams_verified':38,'whole_class_members':12,'runtime_legs':8,'char_values_per_leg':65536,'range':[0,65535],'expected_stdout':'chars=65536,sum=2147450880\n','cli_sha256':d['cli_sha256'],'fixture_sha256':d['fixture_sha256'],'acceptance_scope':'Whole Meet source preserved; original, JADX and Jarde default/all compile and run on JDK8/23. Every char return is checked individually. This does not accept all CF12 scenarios.'}
path=out/'acceptance-root-v1.json'
with path.open('x') as f:f.write(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))
