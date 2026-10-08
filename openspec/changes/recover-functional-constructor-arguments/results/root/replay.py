import json, pathlib, subprocess, tempfile, sys

root = pathlib.Path(__file__).resolve().parents[5]
cli = pathlib.Path(sys.argv[1])
evidence = root / 'openspec/evidence/java-syntax-2026-10-08'
out = root / 'openspec/changes/recover-functional-constructor-arguments/results/root'
out.mkdir(parents=True, exist_ok=True)

def run(args, log):
    p = subprocess.run([str(a) for a in args], capture_output=True)
    log.write_bytes(p.stdout + p.stderr)
    assert p.returncode == 0, (args, p.returncode, log)
    return p.stdout

def render(inp, name, policy, dest):
    p = subprocess.run([str(cli), 'class-source', '--input', str(inp), '--class', name,
                        '--policy', policy, '--format', 'text', '--release', '8'], capture_output=True)
    assert p.returncode in (0, 4), (name, p.returncode, p.stderr[-1000:])
    s = p.stdout.decode()
    assert f'// jarde: presentation of `{name}`' in s and f'class {name}' in s, name
    dest.write_text(s)
    return '\n'.join(line for line in s.splitlines() if not line.lstrip().startswith('//')) + '\n'

for leg, home in [('v8-javac8', '/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'),
                  ('v8', '/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
    jdk = pathlib.Path(home) / 'bin'
    flags = [] if leg == 'v8-javac8' else ['--release', '8']
    logdir = out / leg
    logdir.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='jarde-root-ctor-') as tmp:
        tmp = pathlib.Path(tmp)
        fixture = evidence / 'lambda-constructor-arguments' / leg / 'fixture.jar'
        for name in ['FunctionalConstructors', 'IntBox']:
            s = render(fixture, name, 'plain-jar', logdir / (name + '.text'))
            assert '@bytecode' not in (logdir / (name + '.text')).read_text(), name
            (tmp / (name + '.java')).write_text(s)
        driver = evidence / 'lambda-constructor-arguments/source/Driver.java'
        run([jdk/'javac', *flags, '-d', tmp, tmp/'FunctionalConstructors.java', tmp/'IntBox.java', driver], logdir/'javac.log')
        got = run([jdk/'java', '-Xverify:all', '-cp', tmp, 'Driver'], logdir/'Driver.stdout')
        assert got == (evidence/'lambda-constructor-arguments'/leg/'source.stdout').read_bytes()
        jadxdir = tmp/'jadx'
        jadxdir.mkdir()
        jadx = evidence/'lambda-constructor-arguments'/leg/'jadx'
        run([jdk/'javac', *flags, '-d', jadxdir, jadx/'sources/defpackage/FunctionalConstructors.java', jadx/'sources/defpackage/IntBox.java', jadx/'recompiled/Driver.java'], logdir/'jadx-javac.log')
        got = run([jdk/'java', '-Xverify:all', '-cp', jadxdir, 'defpackage.Driver'], logdir/'jadx-Driver.stdout')
        assert got == (evidence/'lambda-constructor-arguments'/leg/'source.stdout').read_bytes()
    with tempfile.TemporaryDirectory(prefix='jarde-root-timing-') as tmp:
        tmp = pathlib.Path(tmp)
        timing = evidence / 'bound-reference-creation-timing'
        run([jdk/'javac', *flags, '-cp', timing, '-d', tmp, timing/'Driver.java', timing/'StandDriver.java'], logdir/'timing-javac.log')
        for name, driver in [('NoCheck', 'Driver'), ('NoStand', 'StandDriver')]:
            got = run([jdk/'java', '-Xverify:all', '-cp', str(tmp)+':'+str(timing), driver], logdir/(name+'-original.stdout'))
            assert got == b'creation=ok\ninvocation=NPE\n'
            render(timing/(name+'.class'), name, 'single-class', logdir/(name+'.text'))
            text = (logdir/(name+'.text')).read_text()
            assert '@bytecode' in text and 'arg0::start' not in text, name
            p = subprocess.run([str(cli), 'class-source', '--input', str(timing/(name+'.class')), '--class', name, '--policy', 'single-class', '--format', 'json', '--evidence', 'all', '--release', '8'], capture_output=True)
            assert p.returncode in (0, 4)
            (logdir/(name+'.json')).write_bytes(p.stdout)
            record = next(m for m in json.loads(p.stdout)['methods'] if 'make(' in m['text'])
            origins = set()
            for segment in record['outcome']['report']['source_map']['segments']:
                origins.add(segment['origin']['primary']['bci'])
                origins.update(o['bci'] for o in segment['origin']['derived'])
            assert origins == ({0, 3, 4, 5, 10, 13} if name == 'NoCheck' else {0, 1, 6}), (name, origins)
    print(leg + ': full FunctionalConstructors/IntBox replay and both JVM timing negatives passed', flush=True)

legacy = root/'openspec/evidence/java-syntax-2026-10-05/legacy-collections-patrol'
s = render(legacy/'fixture/lg.jar', 'LG', 'plain-jar', out/'LG.text')
def member_text(text, name):
    start = text.find(' '+name+'(')
    assert start >= 0, name
    start = text.rfind('\n', 0, start) + 1
    end = text.find('\n    }', start)
    assert end > start, name
    return text[start:end+6]

before = (evidence/'lambda-constructor-arguments/baseline-LG.txt').read_text()
after = (out/'LG.text').read_text()
assert '@bytecode' in member_text(before, 'pqLambda')
assert '@bytecode' not in member_text(after, 'pqLambda')
for name in ['stackOps', 'dequeOps', 'finalize', 'main']:
    assert member_text(before, name) == member_text(after, name), name+' changed outside the constructor slice'
(out/'LG-unchanged-members.txt').write_text('stackOps/dequeOps/finalize/main: byte-for-byte identical method texts to the frozen main baseline\n')
with tempfile.TemporaryDirectory(prefix='jarde-root-lg-') as tmp:
    tmp = pathlib.Path(tmp)
    (tmp/'LG.java').write_text(s)
    (tmp/'OriginalDriver.java').write_text('public class OriginalDriver { public static void main(String[] a) { System.out.println(java.util.Arrays.toString(LG.pqLambda(new int[]{3,1,2}))); System.out.println(java.util.Arrays.toString(LG.stackOps(new int[]{7,8}))); System.out.println(LG.dequeOps(new int[]{5,6})); } }')
    (tmp/'LGLambda.java').write_text('public class LGLambda {\n'+member_text(s,'pqLambda')+'\n}\n')
    (out/'LGLambda.java').write_text((tmp/'LGLambda.java').read_text())
    (tmp/'LambdaDriver.java').write_text('public class LambdaDriver { public static void main(String[] a) { System.out.println(java.util.Arrays.toString(LGLambda.pqLambda(new int[]{3,1,2}))); } }')
    jadxsrc = tmp/'jadxsrc'
    jadxsrc.mkdir()
    (jadxsrc/'LG.java').write_text((out/'LG-jadx.java').read_text())
    for leg, home in [('v8-javac8', '/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), ('v8', '/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
        jdk = pathlib.Path(home)/'bin'
        flags = [] if leg == 'v8-javac8' else ['--release','8']
        dest = tmp/leg
        dest.mkdir()
        for sub in ['original','recovered','full-failed','jadx-failed']:
            (dest/sub).mkdir()
        p = subprocess.run([str(jdk/'javac'), *flags, '-d', str(dest/'full-failed'), str(tmp/'LG.java')], capture_output=True)
        (out/(leg+'-LG-full-javac.log')).write_bytes(p.stdout+p.stderr)
        assert p.returncode != 0 and b'jarde_refused_body' in p.stderr and b'ArrayList' in p.stderr
        p = subprocess.run([str(jdk/'javac'), *flags, '-d', str(dest/'jadx-failed'), str(jadxsrc/'LG.java')], capture_output=True)
        (out/(leg+'-LG-jadx-javac.log')).write_bytes(p.stdout+p.stderr)
        assert p.returncode != 0 and b'intValue' in p.stderr
        run([jdk/'javac', *flags, '-cp', legacy/'fixture/lg.jar', '-d', dest/'original', tmp/'OriginalDriver.java'], out/(leg+'-LG-original-javac.log'))
        got = run([jdk/'java', '-Xverify:all', '-cp', str(dest/'original')+':'+str(legacy/'fixture/lg.jar'), 'OriginalDriver'], out/(leg+'-LG-original.stdout'))
        assert got == b'[3, 2, 1]\n[8, 8, 1]\n[6, 99, 3]\n'
        run([jdk/'javac', *flags, '-d', dest/'recovered', tmp/'LGLambda.java', tmp/'LambdaDriver.java'], out/(leg+'-LGLambda-javac.log'))
        got = run([jdk/'java', '-Xverify:all', '-cp', dest/'recovered', 'LambdaDriver'], out/(leg+'-LGLambda.stdout'))
        assert got == b'[3, 2, 1]\n'
print('LG pqLambda replay passed on both JDKs; four other methods unchanged; complete Jarde/JADX LG compilation failures retained', flush=True)

known = evidence/'bound-reference-creation-timing/positive'
known_source = render(known/'KnownBoundPositive.class', 'KnownBoundPositive', 'single-class', out/'KnownBoundPositive.text')
assert '@bytecode' not in (out/'KnownBoundPositive.text').read_text()
with tempfile.TemporaryDirectory(prefix='jarde-root-bound-positive-') as tmp:
    tmp = pathlib.Path(tmp)
    (tmp/'KnownBoundPositive.java').write_text(known_source)
    for leg, home in [('v8-javac8', '/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home'), ('v8', '/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home')]:
        jdk = pathlib.Path(home)/'bin'
        flags = [] if leg == 'v8-javac8' else ['--release','8']
        dest = tmp/leg
        dest.mkdir()
        run([jdk/'javac', *flags, '-d', dest, tmp/'KnownBoundPositive.java', known/'PositiveDriver.java'], out/(leg+'-KnownBound-javac.log'))
        got = run([jdk/'java', '-Xverify:all', '-cp', dest, 'PositiveDriver'], out/(leg+'-KnownBound.stdout'))
        assert got == b'bound\n5\n'
print('entry-this and direct String full-class replay passed on both JDKs; javac Class-literal dup/check stays a separate boundary', flush=True)
