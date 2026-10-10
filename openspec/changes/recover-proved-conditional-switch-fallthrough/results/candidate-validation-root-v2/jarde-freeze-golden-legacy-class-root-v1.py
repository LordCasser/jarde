from pathlib import Path
from blake3 import blake3
import struct,hashlib,json
root=Path('/Users/lordcasser/workspace/projects/jarde')
out=Path('/private/tmp/jarde-golden-legacy-class-root-v1');out.mkdir(exist_ok=False)
u2=lambda n:struct.pack('>H',n)
u4=lambda n:struct.pack('>I',n)
utf=lambda s:b'\x01'+u2(len(s))+s
pool=[utf(b'Test'),b'\x07'+u2(1),utf(b'java/lang/Object'),b'\x07'+u2(3),utf(b'method'),utf(b'()V'),utf(b'Code')]
code=bytes.fromhex('a80007a80004b14ba900')
content=u2(1)+u2(1)+u4(len(code))+code+u2(0)+u2(0)
data=u4(0xcafebabe)+u2(0)+u2(50)+u2(8)+b''.join(pool)+u2(0x21)+u2(2)+u2(4)+u2(0)+u2(0)+u2(1)+u2(9)+u2(5)+u2(6)+u2(1)+u2(7)+u4(len(content))+content+u2(0)
assert len(data)==114 and blake3(data).hexdigest()=='dac91477a63e8c8e43acbf4e56916bab8a7415f862314f47eb5e904ef1adc019'
gold=json.loads((root/'tests/fixtures/p2-golden/legacy-clone.json').read_bytes())
assert 'dac91477a63e8c8e43acbf4e56916bab8a7415f862314f47eb5e904ef1adc019' in json.dumps(gold)
fixture=root/'tests/fixtures/p3-conditional-switch-boundaries/legacy-clone/Test.class';fixture.parent.mkdir(exist_ok=False)
fixture.write_bytes(data)
record={'status':'byte-identical-existing-p2-golden-fixture-only','bytes':len(data),'blake3':blake3(data).hexdigest(),'sha256':hashlib.sha256(data).hexdigest(),'fixture':str(fixture),'builder':'tests/p2_golden.rs::jsr_class(50) / jsr_body / simple_class','golden_sha256':hashlib.sha256((root/'tests/fixtures/p2-golden/legacy-clone.json').read_bytes()).hexdigest(),'runtime_executed':False}
(out/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
