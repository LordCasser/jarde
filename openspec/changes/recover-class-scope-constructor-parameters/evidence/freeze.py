from pathlib import Path
import argparse, hashlib, json, shutil, subprocess, tempfile

ROOT=Path(__file__).resolve().parent
JDKS={'corretto8':Path('/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
      'openjdk23':Path('/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')}
JADX=Path('/opt/homebrew/bin/jadx')
POS={'Hold','BoundHold','ArrayHold','WideHold','WideStoredHold','RepeatedHold','MultiHold','UnusedHold','RawNewHold'}
PARTIAL_POS={'PeerNewHold'}
parser=argparse.ArgumentParser(); parser.add_argument('--only',nargs='*'); selected=set(parser.parse_args().only or [])

def run(argv,base):
    p=subprocess.run([str(x) for x in argv],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    base.with_suffix('.stdout').write_bytes(p.stdout); base.with_suffix('.stderr').write_bytes(p.stderr)
    base.with_suffix('.exit').write_text(str(p.returncode)+'\n')
    return p
for leg,jdk in JDKS.items():
  for debug in ('debug','nodebug'):
    flags=['-source','8','-target','8'] if leg=='corretto8' else ['--release','8']
    flags+=['-g'] if debug=='debug' else ['-g:none']
    for src in sorted((ROOT/'fixtures').glob('*/**/*.java')):
      name=src.stem
      if selected and name not in selected: continue
      out=ROOT/'frozen'/leg/debug/name
      if out.exists(): shutil.rmtree(out)
      legacy_driver=out/'driver-classes'
      if legacy_driver.exists(): shutil.rmtree(legacy_driver)
      classes=out/'classes'; classes.mkdir(parents=True,exist_ok=True)
      (out/'source').mkdir(exist_ok=True)
      (out/'source'/src.name).write_bytes(src.read_bytes())
      p=run([jdk/'bin/javac',*flags,'-d',classes,src],out/'javac')
      (out/'kind.txt').write_text(('positive' if name in POS else 'partial_positive' if name in PARTIAL_POS else 'boundary')+'\n')
      if p.returncode: continue
      jar=out/(name+'.jar')
      jp=run([jdk/'bin/jar','cf',jar,'-C',classes,'.'],out/'jar')
      entries=subprocess.run([str(jdk/'bin/jar'),'tf',str(jar)],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
      (out/'jar.contents').write_bytes(entries.stdout); (out/'jar.contents.stderr').write_bytes(entries.stderr)
      assert entries.returncode==0 and name+'.class' in entries.stdout.decode().splitlines()
      assert 'ConstructorDriver.class' not in entries.stdout.decode()
      javap=run([jdk/'bin/javap','-classpath',jar,'-p','-v','-c',name],out/(name+'-javap'))
      (out/(name+'.javap')).write_bytes(javap.stdout)
      with tempfile.TemporaryDirectory(prefix='jarde-ctor-freeze-driver-') as driver_temp:
        driver_dir=Path(driver_temp)
        dp=run([jdk/'bin/javac',*flags,'-d',driver_dir,ROOT/'ConstructorDriver.java'],out/'driver-javac')
        if dp.returncode==0:
          run([jdk/'bin/java','-Xverify:all','-cp',str(classes)+':'+str(driver_dir),'ConstructorDriver',name],out/'reflection-run')
      dx=out/'jadx'; dx.mkdir(exist_ok=True)
      run([JADX,'-d',dx,jar],out/'jadx-decompile')
      resources=dx/'resources'
      if resources.exists(): shutil.rmtree(resources)
      (out/'sha256.json').write_text(json.dumps({**{str(p.relative_to(classes)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(classes.rglob('*.class'))},src.name:hashlib.sha256(src.read_bytes()).hexdigest(),jar.name:hashlib.sha256(jar.read_bytes()).hexdigest()},indent=2)+'\n')
(ROOT/'freeze-summary.json').write_text(json.dumps({'jdk_homes':{k:str(v) for k,v in JDKS.items()},'jadx':str(JADX),'jadx_version':subprocess.run([str(JADX),'--version'],capture_output=True,text=True).stdout.strip(),'cases':sorted(p.name for p in (ROOT/'fixtures').iterdir() if p.is_dir()),'modes':['debug','nodebug']},indent=2)+'\n')
