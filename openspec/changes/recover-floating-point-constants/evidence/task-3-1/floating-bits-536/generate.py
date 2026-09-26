from pathlib import Path
import random, subprocess, hashlib

WORK = Path('/tmp/fbits')
OUT = WORK / 'original'
OUT.mkdir(exist_ok=True)
RECOVERED = WORK / 'recovered'
RECOVERED.mkdir(exist_ok=True)

# --- 536 finite values, same generation as the spelling-hypothesis evidence ---
randomizer = random.Random(20260923)
values = {32: [0, 0x80000000, 1, 0x80000001, 0x7fffff, 0x807fffff, 0x800000, 0x80800000,
               0x3f800000, 0xbf800000, 0x7f7fffff, 0xff7fffff],
          64: [0, 0x8000000000000000, 1, 0x8000000000000001, 0xfffffffffffff, 0x800fffffffffffff,
               0x10000000000000, 0x8010000000000000, 0x3ff0000000000000, 0xbff0000000000000,
               0x7fefffffffffffff, 0xffefffffffffffff]}
for width in (32, 64):
    fraction_bits, maximum = (23, 0xff) if width == 32 else (52, 0x7ff)
    while len(values[width]) < 268:
        bits = randomizer.getrandbits(width)
        if ((bits >> fraction_bits) & maximum) != maximum:
            values[width].append(bits)

def spell(bits, width):
    fraction_bits, bias, digits, suffix = (23, 127, 6, 'f') if width == 32 else (52, 1023, 13, 'd')
    exponent = (bits >> fraction_bits) & (0xff if width == 32 else 0x7ff)
    fraction = bits & ((1 << fraction_bits) - 1)
    assert exponent != (0xff if width == 32 else 0x7ff)
    sign = '-' if bits >> (width - 1) else ''
    if exponent == 0 and fraction == 0:
        return sign + '0.0' + suffix
    if width == 32:
        fraction <<= 1
    return f'{sign}0x{1 if exponent else 0:x}.{fraction:0{digits}x}p{exponent - bias if exponent else 1 - bias}{suffix}'

lines = ['public class FloatingBits536 {']
runner = ['public class FloatingBitsRunner536 {', '  public static void main(String[] args) {']
for width, kind, prefix in ((32, 'float', 'fl'), (64, 'double', 'db')):
    owner, method, suffix = ('Float', 'floatToRawIntBits', '') if width == 32 else ('Double', 'doubleToRawLongBits', 'L')
    for i, bits in enumerate(values[width]):
        lines.append(f'  public static {kind} {prefix}{i}() {{ return {spell(bits, width)}; }}')
        runner.append(f'    System.out.println("{kind}{i}=" + Long.toHexString({owner}.{method}(FloatingBits536.{prefix}{i}())) + "{suffix}");')
lines.append('}')
runner.append('  }')
runner.append('}')
(WORK / 'FloatingBits536.java').write_text('\n'.join(lines) + '\n')
(WORK / 'FloatingBitsRunner536.java').write_text('\n'.join(runner) + '\n')

r = subprocess.run(['javac', '--release', '8', '-g:none', '-d', str(OUT),
                    str(WORK / 'FloatingBits536.java'), str(WORK / 'FloatingBitsRunner536.java')],
                   capture_output=True, text=True)
assert r.returncode == 0, r.stderr
r = subprocess.run(['java', '-Xverify:all', '-cp', str(OUT), 'FloatingBitsRunner536'],
                   capture_output=True, text=True)
assert r.returncode == 0, r.stderr
(OUT / 'runner-output.txt').write_text(r.stdout)
print('original lines:', r.stdout.count('\n'))

# --- patch the frozen FloatingConstants class: fneg/dneg after the canonical NaN loads ---
import struct
base = Path('/Users/lordcasser/workspace/projects/jarde/tests/fixtures/p3-floating-constants/v8/FloatingConstants.class').read_bytes()

def code_attribute(code, max_stack):
    return (len(code) + 12).to_bytes(4, 'big') + max_stack.to_bytes(2, 'big') + b'\0\0' + len(code).to_bytes(4, 'big') + code + b'\0\0\0\0'

def replace_code(data, before, after, max_stack):
    old = code_attribute(before, max_stack)
    assert data.count(old) == 1, f'expected one Code attribute {before.hex()}'
    return data.replace(old, code_attribute(after, max_stack))

patched = replace_code(base, bytes.fromhex('120fae'), bytes.fromhex('120f76ae'), 1)
patched = replace_code(patched, bytes.fromhex('140022af'), bytes.fromhex('14002277af'), 2)
(WORK / 'FloatingConstantsNanFold.class').write_bytes(patched)
print('patched sha256:', hashlib.sha256(patched).hexdigest())
