"""Verifier-valid Java 8 LMF sites without a creation-time receiver null check."""
from pathlib import Path
import struct

HERE = Path(__file__).parent
u2 = lambda n: struct.pack('>H', n)
u4 = lambda n: struct.pack('>I', n)

def fixture(name, constructor):
    pool = []
    def add(tag, data):
        pool.append(bytes([tag]) + data)
        return len(pool)
    def utf(s):
        data = s.encode()
        return add(1, u2(len(data)) + data)
    def cls(s): return add(7, u2(utf(s)))
    def nat(n, d): return add(12, u2(utf(n)) + u2(utf(d)))
    def method(c, n, d): return add(10, u2(c) + u2(nat(n, d)))
    owner, obj = cls(name), cls('java/lang/Object')
    thread, meta = cls('java/lang/Thread'), cls('java/lang/invoke/LambdaMetafactory')
    code_name, bsm_name = utf('Code'), utf('BootstrapMethods')
    factory = method(meta, 'metafactory', '(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodType;Ljava/lang/invoke/MethodHandle;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;')
    factory_handle = add(15, bytes([6]) + u2(factory))
    sam = add(16, u2(utf('()V')))
    impl_handle = add(15, bytes([5]) + u2(method(thread, 'start', '()V')))
    instantiated = add(16, u2(utf('()V')))
    indy = add(18, u2(0) + u2(nat('run', '(Ljava/lang/Thread;)Ljava/lang/Runnable;')))
    ctor = method(thread, '<init>', '(Ljava/lang/Runnable;)V')
    make_name = utf('make')
    result = 'Thread' if constructor else 'Runnable'
    make_desc = utf(f'(Ljava/lang/Thread;)Ljava/lang/{result};')
    dynamic = bytes([0x2a, 0xba]) + u2(indy) + bytes([0, 0])
    code = (bytes([0xbb]) + u2(thread) + bytes([0x59]) + dynamic + bytes([0xb7]) + u2(ctor) + bytes([0xb0])) if constructor else dynamic + bytes([0xb0])
    code_attr = u2(3 if constructor else 1) + u2(1) + u4(len(code)) + code + u2(0) + u2(0)
    method_info = u2(9) + u2(make_name) + u2(make_desc) + u2(1) + u2(code_name) + u4(len(code_attr)) + code_attr
    bsm = u2(1) + u2(factory_handle) + u2(3) + u2(sam) + u2(impl_handle) + u2(instantiated)
    return bytes.fromhex('cafebabe') + u2(0) + u2(52) + u2(len(pool)+1) + b''.join(pool) + u2(33) + u2(owner) + u2(obj) + u2(0) + u2(0) + u2(1) + method_info + u2(1) + u2(bsm_name) + u4(len(bsm)) + bsm

for name, constructor in [('NoCheck', True), ('NoStand', False)]:
    (HERE / f'{name}.class').write_bytes(fixture(name, constructor))
