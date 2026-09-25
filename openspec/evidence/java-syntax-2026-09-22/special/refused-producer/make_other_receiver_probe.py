from pathlib import Path
import struct
src=Path('tests/fixtures/p3-special-dispatch/v8/SpecialProbe.class'); b=bytearray(src.read_bytes()); p=8
u2=lambda off:struct.unpack_from('>H',b,off)[0]
u4=lambda off:struct.unpack_from('>I',b,off)[0]
n=u2(p);p+=2;cp={};i=1
while i<n:
 tag=b[p];p+=1
 if tag==1:
  size=u2(p);p+=2;cp[i]=(tag,bytes(b[p:p+size]).decode());p+=size
 elif tag in [7,8,16,19,20]:cp[i]=(tag,u2(p));p+=2
 elif tag in [9,10,11,12,17,18]:cp[i]=(tag,u2(p),u2(p+2));p+=4
 elif tag in [3,4]:p+=4
 elif tag in [5,6]:p+=8;i+=1
 elif tag==15:p+=3
 else:raise ValueError(tag)
 i+=1
p+=6;p+=2+2*u2(p)
def skip_attrs(p,count):
 for _ in range(count):p+=6+u4(p+2)
 return p
fields=u2(p);p+=2
for _ in range(fields):p=skip_attrs(p+8,u2(p+6))
methods=u2(p);p+=2
ref=next(i for i,v in cp.items() if v[0]==10 and cp[cp[v[2]][1]][1]=='valueWith')
for _ in range(methods):
 start=p;name=cp[u2(p+2)][1];ac=u2(p+6);p+=8
 if name=='privateHelper':pass
 for _ in range(ac):
  attr=cp[u2(p)][1];length=u4(p+2)
  if name=='callOtherPrivate' and attr=='Code':
   code=p+14;assert b[code+2]==0xb7;struct.pack_into('>H',b,code+3,ref)
  p+=6+length
out=Path('/tmp/jarde-special-other-receiver');out.mkdir(exist_ok=True);(out/'SpecialProbe.class').write_bytes(b)
print('patch current-class non-private special target index',ref)
