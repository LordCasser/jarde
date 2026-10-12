"""Deterministically write the tiny classfile shapes needed by SSA negative tests."""
from pathlib import Path
import struct

u1 = lambda x: struct.pack('>B', x)
u2 = lambda x: struct.pack('>H', x)
u4 = lambda x: struct.pack('>I', x)
cp = [None]
def utf8(s):
    b = s.encode('utf-8')
    cp.append(b'\x01' + u2(len(b)) + b)
    return len(cp)-1
def cls(name):
    i=utf8(name); cp.append(b'\x07'+u2(i)); return len(cp)-1
def name_type(name, desc):
    n=utf8(name); d=utf8(desc); cp.append(b'\x0c'+u2(n)+u2(d)); return len(cp)-1
def methodref(owner, name, desc):
    c=cls(owner); nt=name_type(name, desc); cp.append(b'\x0a'+u2(c)+u2(nt)); return len(cp)-1
def fieldref(owner, name, desc):
    c=cls(owner); nt=name_type(name, desc); cp.append(b'\x09'+u2(c)+u2(nt)); return len(cp)-1

this_class=cls('reference/LoadNegatives')
super_class=cls('java/lang/Object')
code_name=utf8('Code')
obj_init=methodref('java/lang/Object','<init>','()V')
multi_name=utf8('multi'); obj_desc=utf8('(Ljava/lang/Object;)Ljava/lang/Object;')
effect_name=utf8('effect'); void_desc=utf8('()V')
cross_name=utf8('crossEffect')
effect_ref=methodref('reference/LoadNegatives','effect','()V')
effects_field=utf8('effects'); int_desc=utf8('I')
effects_ref=fieldref('reference/LoadNegatives','effects','I')
def code_attr(max_stack,max_locals,code):
    body=u2(max_stack)+u2(max_locals)+u4(len(code))+code+u2(0)+u2(0)
    return u2(code_name)+u4(len(body))+body
def method(flags,name,desc,code,max_stack,max_locals):
    return u2(flags)+u2(name)+u2(desc)+u2(1)+code_attr(max_stack,max_locals,code)
# multi: aload_0; dup; astore_1; areturn. The same reference SSA value feeds
# the local store and return, so its use list has two real consumers.
multi=method(0x0009,multi_name,obj_desc,b'\x2a\x59\x4c\xb0',2,2)
effect=method(0x0009,effect_name,void_desc,b'\x04\xb3'+u2(effects_ref)+b'\xb1',1,0)
# crossEffect: aload_0; invokestatic effect()V; astore_1; aload_1; areturn.
# The original reference remains on stack across an observable call before its store.
cross=method(0x0009,cross_name,obj_desc,b'\x2a\xb8'+u2(effect_ref)+b'\x4c\x2b\xb0',1,2)
body=(u4(0xcafebabe)+u2(0)+u2(52)+u2(len(cp))+
      b''.join(x for x in cp[1:])+u2(0x0021)+u2(this_class)+u2(super_class)+
      u2(0)+u2(1)+u2(0x0008)+u2(effects_field)+u2(int_desc)+u2(0)+u2(3)+multi+effect+cross+u2(0))
Path(__file__).with_name('LoadNegatives.class').write_bytes(body)
