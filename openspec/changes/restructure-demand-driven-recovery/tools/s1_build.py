#!/usr/bin/env python3
"""串行构建 S1 检查点，保存命令与来源；默认栈、空间守卫及 finally 清理。"""
import hashlib,json,os,shutil,signal,subprocess,sys,time
from pathlib import Path
root=Path.cwd(); out=Path(sys.argv[1]).resolve(); target=Path(sys.argv[2]).resolve(); binary_dir=Path(sys.argv[3]).resolve(); plan=json.loads(Path(sys.argv[4]).read_text())
assert not target.exists(),target
out.mkdir(parents=True,exist_ok=True);binary_dir.mkdir(parents=True,exist_ok=True)
def pin(p):
    data=p.read_bytes();return dict(path=str(p),bytes=len(data),sha256=hashlib.sha256(data).hexdigest())
files=[root/p for p in subprocess.check_output(['rg','--files','-g','*.rs','-g','Cargo.toml','-g','Cargo.lock','-g','!openspec/**'],text=True).splitlines()]
before=[pin(p) for p in sorted(files)];records=[];binaries=[];child=None
(out/'source-before.json').write_text(json.dumps(before,indent=2)+'\n')
env=dict(os.environ,CARGO_TARGET_DIR=str(target),CARGO_BUILD_JOBS='1',CARGO_INCREMENTAL='0',CARGO_PROFILE_RELEASE_DEBUG='0',CARGO_PROFILE_RELEASE_STRIP='none');env.pop('RUST_MIN_STACK',None)
try:
    for task in plan:
        name=task['id'];cmd=task['command'];assert cmd[0]=='cargo'
        started=time.monotonic();peak=0;stop=None
        with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:
            child=subprocess.Popen(cmd,cwd=root,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
            while child.poll() is None:
                size=0
                for p in target.rglob('*'):
                    try:
                        if p.is_file():size+=p.stat().st_size
                    except FileNotFoundError:pass
                peak=max(peak,size)
                if size>1<<30 or shutil.disk_usage(target.parent).free<5<<30:stop='target_or_free_space_guard'
                if time.monotonic()-started>900:stop='build_timeout'
                if stop:
                    os.killpg(child.pid,signal.SIGTERM);child.wait(timeout=15);break
                time.sleep(1)
            rc=child.wait()
        records.append(dict(id=name,command=cmd,exit_code=rc,stop=stop,seconds=time.monotonic()-started,peak_target_bytes=peak))
        print(json.dumps(records[-1]),flush=True)
        assert rc==0 and stop is None,records[-1]
        for line in (out/(name+'.stdout')).read_text().splitlines():
            try:item=json.loads(line)
            except json.JSONDecodeError:continue
            if item.get('executable'):
                dest=binary_dir/(name+'-'+item['target']['name']);shutil.copy2(item['executable'],dest);binaries.append(pin(dest))
finally:
    if child is not None and child.poll() is None:
        os.killpg(child.pid,signal.SIGTERM);child.wait(timeout=15)
    shutil.rmtree(target,ignore_errors=True)
    after=[pin(p) for p in sorted(files)]
    receipt=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),commands=records,binaries=binaries,target=str(target),target_absent=not target.exists(),sources_unchanged=before==after,source_before=pin(out/'source-before.json'),rustc=subprocess.check_output(['rustc','-Vv'],text=True))
    (out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    assert before==after,'source mutation during build'
