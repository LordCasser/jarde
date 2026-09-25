from pathlib import Path
import os,subprocess,json
R=Path('/Users/lordcasser/workspace/projects/jarde');O=Path(__file__).parent
env=os.environ.copy();env.update(CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',RUST_TEST_THREADS='1')
checks=[
 ('java-package',['cargo','test','-p','jarde-java','--locked']),
 ('adjacent',['cargo','test','--locked','--test','p3_final_static','--test','p3_declaration_handoff','--test','p3_alias_field','--test','p3_field_increment','--test','p3_new_value','--test','p3_invocation_arguments','--test','p3_eval_context','--test','p3_recovery_entry']),
 ('final-static-jdk',['cargo','test','--locked','--test','p3_final_static','--','--ignored']),
 ('census-before',['cargo','test','-p','jarde-reader','--locked','repository_class_fixtures_validate_without_false_target_rejections','--','--nocapture']),
 ('deferred-red',['cargo','test','--locked','--test','p3_deferred_value_order']),
 ('floating-red',['cargo','test','--locked','--test','p3_floating_constants']),
 ('bitwise-red',['cargo','test','--locked','--test','p3_bitwise']),
 ('instanceof-red',['cargo','test','--locked','--test','p3_instanceof']),
 ('fmt',['cargo','fmt','--all','--check']),
 ('clippy',['cargo','clippy','-p','jarde-java','--all-targets','--locked','--','-D','warnings']),
]
results=[]
for name,cmd in checks:
 with (O/(name+'.log')).open('w')as f:
  f.write(' '.join(cmd)+'\n');f.flush();p=subprocess.run(cmd,cwd=R,env=env,stdout=f,stderr=subprocess.STDOUT)
 results.append({'check':name,'exit':p.returncode});(O/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
 print(name,p.returncode,flush=True)
