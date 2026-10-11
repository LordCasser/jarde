#!/usr/bin/env python3
"""Re-run frozen replay verification, then validate report numeric and source-map envelopes."""
import contextlib,hashlib,io,json,runpy,sys
from pathlib import Path
BASE=Path(__file__).resolve().parent
VERIFIER=BASE/'verify-conditional-replay.py'
VERIFIER_SHA='1f9badb69cf940aebbb4e622b1da7ac3c11d7068928047925b49eb2b014b7ebf'
def need(ok,message):
    if not ok:raise ValueError(message)
def main():
    need(hashlib.sha256(VERIFIER.read_bytes()).hexdigest()==VERIFIER_SHA,'reviewed replay verifier changed')
    module=runpy.run_path(str(VERIFIER));captured=io.StringIO()
    with contextlib.redirect_stdout(captured):module['main']()
    accepted=json.loads(captured.getvalue())
    execution=Path(sys.argv[sys.argv.index('--execution')+1]);ex=json.loads(execution.read_bytes())
    reports=segments=anchors=counter_slots=0
    for row in ex['case_rows']:
        for leg,lr in row['legs'].items():
            for profile,pr in lr['profiles'].items():
                for binding in pr['report_bindings']:
                    doc=json.loads(Path(binding['report_path']).read_bytes());reports+=1
                    for label,usage in module['usage_slots'](doc).items():
                        counter_slots+=1
                        need(all(type(value) is int and value>=0 for value in usage.values()),'non-integer/negative usage counter '+binding['report_path']+'/'+label)
                    for method in doc['methods']:
                        report=method.get('outcome',{}).get('report')
                        if report is None:continue
                        text=report['text'].encode('utf-8')
                        for segment in report['source_map']['segments']:
                            start,end=segment['start'],segment['end']
                            need(type(start) is int and type(end) is int and 0<=start<end<=len(text),'invalid UTF-8 source-map byte bounds')
                            text[:start].decode('utf-8');text[start:end].decode('utf-8');segments+=1
                        item=method['item']
                        if row['case']=='TestSwitchWithFallThroughCase.test' and item['name']['escaped']=='test' and item['descriptor']['escaped']=='(IZZ)Ljava/lang/String;':
                            need(method['outcome']['kind']=='recovered' and report['quality']=='structured' and report['representation']=='java' and report['content']=='contains_statements' and report['fallbacks']==[],'conditional target is not full Structured Java recovery')
                            need(report['execution']['status']=='complete','conditional target recovery stopped');anchors+=1
    need(anchors==4,'exact dual-JDK/default-all conditional anchor inventory changed')
    print(json.dumps({**accepted,'envelope_checks':{'report_count':reports,'usage_slots':counter_slots,'utf8_source_segments':segments,'structured_anchor_profiles':anchors}},indent=2))
if __name__=='__main__':
    try:main()
    except Exception as error:
        print('replay envelope verification failed: '+str(error),file=sys.stderr);raise
