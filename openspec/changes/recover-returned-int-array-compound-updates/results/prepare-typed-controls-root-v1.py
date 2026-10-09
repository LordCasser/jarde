#!/usr/bin/env python3
"""Verify descriptor-only operand controls; never invoke any mutated target method."""
import hashlib,json,os,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
BASE=HERE.parents[1]/"recover-nested-int-array-compound-updates"/"results"/"returned-next-baseline-v1"
OUT=HERE/"typed-controls-root-v1"

def sha(data):return hashlib.sha256(data).hexdigest()
def file(path):
    data=path.read_bytes()
    return {"path":str(path),"bytes":len(data),"sha256":sha(data)}

def main():
    assert not OUT.exists();OUT.mkdir()
    baseline=json.loads((BASE/"manifest.json").read_bytes())
    env=os.environ.copy()
    for name in ("JAVA_TOOL_OPTIONS","_JAVA_OPTIONS","JDK_JAVA_OPTIONS","CLASSPATH"):env.pop(name,None)
    commands=[]
    def run(label,argv,home):
        local_env=env.copy();local_env["JAVA_HOME"]=home
        p=subprocess.run(argv,capture_output=True,env=local_env,timeout=60)
        stdout=OUT/(label+".stdout");stderr=OUT/(label+".stderr")
        stdout.write_bytes(p.stdout);stderr.write_bytes(p.stderr)
        row={"label":label,"argv":argv,"java_home":home,"exit_code":p.returncode,"stdout":file(stdout),"stderr":file(stderr)}
        commands.append(row)
        if p.returncode:raise RuntimeError(f"{label} exit {p.returncode}")
        return p.stdout
    source=OUT/"VerifyOnly.java"
    source.write_text('public final class VerifyOnly { public static void main(String[] args) throws Exception { Class<?> c=Class.forName("ReturnedIntArrayUpdates"); for(java.lang.reflect.Method m:c.getDeclaredMethods()) if(m.getName().equals("scalar")) { Class<?>[] p=m.getParameterTypes(); System.out.println("ok:"+p[1].getName()+":"+p[2].getName()+":"+m.getReturnType().getName()); } } }\n')
    variants={"original":("([III)I","int","int"),"boolean-index":("([IZI)I","boolean","int"),"boolean-rhs":("([IIZ)I","int","boolean"),"char-index":("([ICI)I","char","int"),"char-rhs":("([IIC)I","int","char")}
    cases=[]
    for i,tools in enumerate(baseline["jdk_tools"]):
        home=str(Path(tools["java"]["path"]).parent.parent)
        for tool in tools.values():assert file(Path(tool["path"]))["sha256"]==tool["sha256"]
        leg=("javac8","javac23")[i]
        original=BASE/baseline["inputs"][i]["path"];raw=original.read_bytes()
        assert sha(raw)==baseline["inputs"][i]["sha256"]
        needle=b"\x01\x00\x07([III)I";assert raw.count(needle)==1
        runner_classes=OUT/(leg+"-verifier");runner_classes.mkdir()
        empty=OUT/(leg+"-empty");empty.mkdir()
        run(leg+"-compile-verifier",[tools["javac"]["path"],"-source","8","-target","8","-g:none","-classpath",str(empty),"-sourcepath",str(empty),"-d",str(runner_classes),str(source)],home)
        for variant,(descriptor,index_type,rhs_type) in variants.items():
            folder=OUT/(leg+"-"+variant);folder.mkdir()
            class_path=folder/"ReturnedIntArrayUpdates.class"
            changed=raw.replace(needle,b"\x01\x00\x07"+descriptor.encode(),1)
            assert len(changed)==len(raw)
            differences=[n for n,(a,b) in enumerate(zip(raw,changed)) if a!=b]
            assert len(differences)==(0 if variant=="original" else 1)
            class_path.write_bytes(changed)
            expected=f"ok:{index_type}:{rhs_type}:int\n".encode()
            output=run(leg+"-"+variant+"-load-only",[tools["java"]["path"],"-Xverify:all","-cp",os.pathsep.join((str(runner_classes),str(folder))),"VerifyOnly"],home)
            assert output==expected
            cases.append({"leg":leg,"variant":variant,"descriptor":descriptor,"input":file(class_path),"base":file(original),"differing_byte_offsets":differences,"expected_stdout":expected.decode(),"load_only_success":True,"target_method_invoked":False})
    result={"schema":"returned-array-typed-controls-root-v1","scope":"JVM legality and exact descriptor-only mutation only; no decompiler or runtime semantic success claimed","runner_sha256":sha(Path(__file__).read_bytes()),"verify_only_source":file(source),"baseline_manifest_sha256":sha((BASE/"manifest.json").read_bytes()),"cases":cases,"commands":commands,"jdk_tools":baseline["jdk_tools"]}
    (OUT/"manifest.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
    print(json.dumps({"cases":len(cases),"commands":len(commands),"load_only_success":all(c["load_only_success"] for c in cases),"target_method_invoked":False}))

if __name__=="__main__":main()
