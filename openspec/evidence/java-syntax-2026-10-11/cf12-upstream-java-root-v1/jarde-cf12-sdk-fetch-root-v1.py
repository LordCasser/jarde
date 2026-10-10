import concurrent.futures, hashlib, json, shutil, urllib.request, datetime
from pathlib import Path
OUT=Path('/private/tmp/jarde-cf12-test-sdk-root-v1'); OUT.mkdir(exist_ok=False)
COORDS=[('org.junit.platform','junit-platform-console-standalone','1.14.4'),('org.assertj','assertj-core','3.27.7'),('net.bytebuddy','byte-buddy','1.18.3'),('org.apache.commons','commons-lang3','3.20.0'),('org.jetbrains','annotations','26.1.0'),('org.eclipse.jdt','ecj','3.33.0')]
def fetch(coord):
 g,a,v=coord; stem=f'{a}-{v}'; base='https://repo.maven.apache.org/maven2/'+g.replace('.','/')+'/'+a+'/'+v+'/'+stem
 assert shutil.disk_usage(OUT).free >= 5*1024**3
 result={'coordinate':':'.join(coord),'files':[]}
 for suffix in ('.pom','.jar','.jar.sha1'):
  url=base+suffix
  with urllib.request.urlopen(url, timeout=35) as r:
   data=r.read(20*1024**2+1); status=r.status
  assert status==200 and len(data)<=20*1024**2
  p=OUT/(stem+suffix); p.write_bytes(data)
  result['files'].append({'url':url,'path':str(p),'status':status,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
 expected=(OUT/(stem+'.jar.sha1')).read_text().strip().split()[0]
 actual=hashlib.sha1((OUT/(stem+'.jar')).read_bytes()).hexdigest()
 assert actual==expected,(coord,actual,expected)
 result['upstream_sha1_verified']=actual
 return result
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool: results=list(pool.map(fetch,COORDS))
(OUT/'manifest.json').write_text(json.dumps({'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'artifacts':results},indent=2)+'\n')
print(json.dumps({'out':str(OUT),'artifacts':len(results),'total_jar_bytes':sum(p.stat().st_size for p in OUT.glob('*.jar'))}))
