from pathlib import Path
import hashlib, json, subprocess

ROOT=Path(__file__).resolve().parent
CASES={
'Hold': '''public class Hold<T> { public T v; public Hold(T v) { this.v = v; } }''',
'BoundHold': '''public class BoundHold<T extends Number> { public T v; public BoundHold(T v) { this.v = v; } }''',
'ArrayHold': '''public class ArrayHold<T> { public T[] v; public ArrayHold(T[] v) { this.v = v; } }''',
'WideHold': '''public class WideHold<T> { public T v; public WideHold(long sequence, double weight, T v) { this.v = v; } }''',
'RepeatedHold': '''public class RepeatedHold<T> { public T first; public T second; public RepeatedHold(T v) { this.first=v; this.second=v; } }''',
'WideStoredHold': '''public class WideStoredHold<T> { public long sequence; public double weight; public T v; public WideStoredHold(long sequence, double weight, T v) { this.sequence=sequence; this.weight=weight; this.v=v; } }''',
'MultiHold': '''public class MultiHold<T, U extends CharSequence> { public T first; public U second; public MultiHold(T first, U second) { this.first=first; this.second=second; } }''',
'UnusedHold': '''public class UnusedHold<T> { public T v; public UnusedHold(T ignored) { } }''',
'RawNewHold': '''public class RawNewHold<T> { public T v; public RawNewHold(T v) { this.v = v; } @SuppressWarnings("rawtypes") public static RawNewHold make(Object value) { return new RawNewHold(value); } }''',
'PeerNewHold': '''public class PeerNewHold<T> { public T v; public PeerNewHold(T v) { this.v = v; PeerNewHold<T> peer = new PeerNewHold<T>(v, true); } public PeerNewHold(T v, boolean peer) { this.v = v; } }''',
'ObjectHold': '''public class ObjectHold<T> { public T v; public ObjectHold(Object v) { this.v=(T)v; } }''',
'ErasedCastHold': '''public class ErasedCastHold<T> { public T v; public ErasedCastHold(Object v) { this.v=(T)v; } }''',
'RewrittenHold': '''public class RewrittenHold<T> { public T v; @SuppressWarnings("unchecked") public RewrittenHold(T v) { v=(T)String.valueOf(v); this.v=v; } }''',
'PhiHold': '''public class PhiHold<T> { public T v; public PhiHold(T v, boolean choose) { this.v=choose ? v : null; } }''',
'CallHold': '''public class CallHold<T> { public T v; public CallHold(T v) { this.v=identity(v); } private T identity(T x) { return x; } }''',
'ShadowHold': '''public class ShadowHold<T> { public T v; private T shadow(Object x) { return (T)x; } public <T> ShadowHold(T v) { this.v=shadow(v); } }''',
'CrossHold': '''public class CrossHold<T, U> { public T v; public CrossHold(U v) { this.v=(T)v; } }''',
'ParentCtorHold': '''public class ParentCtorHold<T> extends java.util.Vector<T> { public T v; public ParentCtorHold(T v) { super(1); this.v=v; } }''',
'ThisDelegateHold': '''public class ThisDelegateHold<T> { public T v; public ThisDelegateHold(T v) { this(v, 0); } public ThisDelegateHold(T v, int ignored) { this.v=v; } }''',
'ExceptionHold': '''public class ExceptionHold<T> { public T v; public ExceptionHold(T v) { try { this.v=maybe(v); } catch (RuntimeException ex) { this.v=null; } } private T maybe(T x) { return x; } }''',
}
POSITIVE={'Hold','BoundHold','ArrayHold','WideHold','WideStoredHold','RepeatedHold','MultiHold','UnusedHold','RawNewHold'}
PARTIAL_POSITIVE={'PeerNewHold'}
for name,source in CASES.items():
    d=ROOT/'fixtures'/name; d.mkdir(parents=True,exist_ok=True)
    (d/(name+'.java')).write_text(source+'\n')
    (d/'kind.txt').write_text(('positive' if name in POSITIVE else 'partial_positive' if name in PARTIAL_POSITIVE else 'boundary')+'\n')
REASONS={
'PeerNewHold':'partial positive: the two-argument constructor parameter is class-bound T, while the one-argument constructor writes an erased Object into the same field after constructing a peer instance',
'ObjectHold':'constructor formal parameter is Object, not class variable T',
'ErasedCastHold':'source unchecked cast from Object to T has no runtime cast evidence',
'RewrittenHold':'constructor parameter is reassigned before the field write',
'PhiHold':'field RHS joins parameter and null through a conditional value',
'CallHold':'field RHS comes from a method call',
'ShadowHold':'constructor type variable T shadows class type variable T; declaration identity differs',
'CrossHold':'constructor parameter belongs to class variable U while field belongs to T; body casts between erased variables',
'ParentCtorHold':'constructor calls a non-Object superclass constructor',
'ThisDelegateHold':'one constructor delegates with this(...) to another constructor',
'ExceptionHold':'field write is enclosed by exception handling and a method call',
}
(ROOT/'manifest.json').write_text(json.dumps({n:{'kind':'positive' if n in POSITIVE else 'partial_positive' if n in PARTIAL_POSITIVE else 'boundary','boundary':REASONS.get(n),'source':f'fixtures/{n}/{n}.java'} for n in CASES},indent=2)+'\n')
