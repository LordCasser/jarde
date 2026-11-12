"""Remove only parameterTarget and main method_info records; no CP or retained byte edits."""
from pathlib import Path
import struct,hashlib,json
F=Path(__file__).resolve().parent.parent
def digest(b): return hashlib.sha256(b).hexdigest()
def derive(data):
 p=8
 def u2():
  nonlocal p
  x=struct.unpack_from('>H',data,p)[0];p+=2;return x
 def u4():
  nonlocal p
  x=struct.unpack_from('>I',data,p)[0];p+=4;return x
 cp={}; count=u2();i=1
 while i<count:
  tag=data[p];p+=1
  if tag==1:
   n=u2();cp[i]=(tag,data[p:p+n].decode());p+=n
  elif tag in (3,4):cp[i]=(tag,data[p:p+4]);p+=4
  elif tag in (5,6):cp[i]=(tag,data[p:p+8]);p+=8;i+=1
  elif tag in (7,8,16,19,20):cp[i]=(tag,u2())
  elif tag in (9,10,11,12,18,17):cp[i]=(tag,u2(),u2())
  elif tag==15:cp[i]=(tag,data[p],struct.unpack_from('>H',data,p+1)[0]);p+=3
  else:raise AssertionError(tag)
  i+=1
 cp_end=p;p+=6;n=u2();p+=2*n
 def attributes():
  nonlocal p
  n=u2();out=[]
  for _ in range(n):
   name=cp[u2()][1];size=u4();out.append((name,data[p:p+size]));p+=size
  return out
 for _ in range(u2()):p+=6;attributes()
 count_pos=p;n=u2();records=[]
 for _ in range(n):
  start=p;u2();name=cp[u2()][1];desc=cp[u2()][1];attrs=attributes();records.append((name,desc,start,p,attrs))
 suffix=p; keep=[x for x in records if x[0] not in ('main','parameterTarget')]
 assert [x[0] for x in keep]==['<init>','guardIfFirst']
 # Frozen retained code invokes only Object/FileReader constructors/read/close.
 # Resolve every CP method ref reachable from retained fixed-width bytecode, not stale CP entries.
 refs=[]
 lengths={0x2a:1,0x2b:1,0x2c:1,0x2d:1,0x4c:1,0x4d:1,0x4e:1,0x03:1,0x02:1,0x04:1,0x3d:1,0x3e:1,0x1c:1,0x1d:1,0x60:1,0x59:1,0xac:1,0xb1:1,0xbf:1,0xbb:3,0xb7:3,0xb6:3,0x9e:3,0x9f:3,0xa7:3,0x3a:2,0x19:2}
 for name,desc,start,end,attrs in keep:
  for attr,raw in attrs:
   if attr!='Code':continue
   code=raw[8:8+struct.unpack_from('>I',raw,4)[0]];q=0
   while q<len(code):
    op=code[q];assert op in lengths,(name,q,hex(op))
    if op in (0xb6,0xb7):
     idx=struct.unpack_from('>H',code,q+1)[0];ref=cp[idx];nt=cp[ref[2]];owner=cp[cp[ref[1]][1]][1];member=cp[nt[1]][1];descriptor=cp[nt[2]][1]
     assert member not in ('main','parameterTarget');refs.append([name,q,owner,member,descriptor])
    q+=lengths[op]
 result=data[:count_pos]+struct.pack('>H',len(keep))+b''.join(data[x[2]:x[3]] for x in keep)+data[suffix:]
 assert result[:count_pos]==data[:count_pos] and result[-len(data[suffix:]):]==data[suffix:]
 return result,dict(original_sha256=digest(data),derivative_sha256=digest(result),cp_prefix_sha256=digest(data[:cp_end]),class_attributes_sha256=digest(data[suffix:]),retained=[dict(name=x[0],descriptor=x[1],sha256=digest(data[x[2]:x[3]])) for x in keep],removed=[x[0] for x in records if x not in keep],retained_method_references=refs,scope='Input derivative only. Full generated class text must remain unchanged.')
audit={}
for leg in ['v8','v8-javac8']:
 data=(F/leg/'ProbeControls.class').read_bytes();out,report=derive(data);dest=F/'contract'/leg;dest.mkdir(exist_ok=True);(dest/'ProbeControls.class').write_bytes(out);audit[leg]=report
(F/'contract/derivative-audit.json').write_text(json.dumps(audit,indent=2)+'\n')
print('Byte-preserving derivative prepared and retained invocation closure audited; no JVM executed.')
