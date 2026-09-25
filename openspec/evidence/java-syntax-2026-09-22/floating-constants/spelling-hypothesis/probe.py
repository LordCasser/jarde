from pathlib import Path
import random,subprocess,json
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
WORK=Path('/tmp/jarde-floating-spelling');WORK.mkdir(exist_ok=True)
OUT=ROOT/'openspec/evidence/java-syntax-2026-09-22/floating-constants/spelling-hypothesis';OUT.mkdir(exist_ok=True)
def spell(bits,width):
 fraction_bits,bias,digits,suffix=(23,127,6,'f') if width==32 else (52,1023,13,'d')
 exponent=(bits>>fraction_bits)&(0xff if width==32 else 0x7ff)
 fraction=bits&((1<<fraction_bits)-1)
 assert exponent!=(0xff if width==32 else 0x7ff)
 sign='-' if bits>>(width-1) else ''
 # This is an independently tested spelling hypothesis, not jarde production output.
 if exponent==0 and fraction==0:return sign+'0.0'+suffix
 if width==32:fraction<<=1
 return f'{sign}0x{1 if exponent else 0:x}.{fraction:0{digits}x}p{exponent-bias if exponent else 1-bias}{suffix}'
randomizer=random.Random(20260923)
values={32:[0,0x80000000,1,0x80000001,0x7fffff,0x807fffff,0x800000,0x80800000,0x3f800000,0xbf800000,0x7f7fffff,0xff7fffff],64:[0,0x8000000000000000,1,0x8000000000000001,0xfffffffffffff,0x800fffffffffffff,0x10000000000000,0x8010000000000000,0x3ff0000000000000,0xbff0000000000000,0x7fefffffffffffff,0xffefffffffffffff]}
for width in [32,64]:
 while len(values[width])<268:
  bits=randomizer.getrandbits(width);fraction_bits=23 if width==32 else 52;maximum=0xff if width==32 else 0x7ff
  if ((bits>>fraction_bits)&maximum)!=maximum:values[width].append(bits)
lines=[]
for width in [32,64]:
 for i,bits in enumerate(values[width]):
  owner,method,kind,suffix=('Float','floatToRawIntBits','int','') if width==32 else ('Double','doubleToRawLongBits','long','L')
  lines.append(f' if({owner}.{method}({spell(bits,width)}) != 0x{bits:0{width//4}x}{suffix}) throw new AssertionError("{width}:{i}");')
java='public class FloatingSpellingProbe { public static void main(String[] args){\n'+'\n'.join(lines)+'\nSystem.out.println("passed='+str(len(lines))+'");}}\n'
(WORK/'FloatingSpellingProbe.java').write_text(java);(OUT/'FloatingSpellingProbe.java').write_text(java)
for name,command in [('javac.log',['javac','--release','8','-g:none','-d',str(WORK),str(WORK/'FloatingSpellingProbe.java')]),('execution.txt',['java','-Xverify:all','-cp',str(WORK),'FloatingSpellingProbe'])]:
 p=subprocess.run(command,capture_output=True,text=True,timeout=30);(OUT/name).write_text(p.stdout+p.stderr);assert p.returncode==0
(OUT/'probe.py').write_text(Path(__file__).read_text())
print(json.dumps({'exact_raw_bit_checks':len(lines),'production_changed':False}))
