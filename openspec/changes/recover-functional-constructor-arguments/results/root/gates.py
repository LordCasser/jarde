import os, pathlib, re, subprocess, sys, time
root = pathlib.Path(__file__).resolve().parents[5]
cwd = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else root
logs = root/'openspec/changes/recover-functional-constructor-arguments/results/root/gates'
logs.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', RUST_TEST_THREADS='1', CARGO_TERM_COLOR='never', CARGO_TARGET_DIR=str(root/'target'))
flags = re.findall(r'-A clippy::\w+', (root/'.github/workflows/ci.yml').read_text())
steps = [
    ('fmt', ['cargo','fmt','--all','--','--check'], {}),
    ('clippy', ['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--', *[arg for flag in flags for arg in flag.split()], '-D','warnings'], {}),
    ('seed-1569', ['cargo','test','--workspace','--all-targets','--all-features','--locked','--no-fail-fast'], {'PROPTEST_RNG_SEED':'5350648285461741569'}),
    ('seed-1570', ['cargo','test','--workspace','--all-targets','--all-features','--locked'], {'PROPTEST_RNG_SEED':'5350648285461741570'}),
    ('p3-ignored', ['cargo','test','--test','p3_execution_comparison','--locked','--','--ignored'], {}),
    ('functional-ignored', ['cargo','test','-p','jarde-cli','--test','json_cli','--locked','--','--ignored','--exact','functional_constructor_arguments_replay_the_complete_class_on_both_jdks'], {}),
    ('bound-ignored', ['cargo','test','--test','recover_proved_nonnull_bound_receivers','--locked','--','--ignored','--exact','every_stripped_anchor_answers_what_its_class_answers'], {}),
    ('cli-build', ['cargo','build','-p','jarde-cli','--locked'], {}),
]
for name, args, extra in steps:
    start = time.monotonic()
    print('START '+name, flush=True)
    with (logs/(name+'.log')).open('w') as log:
        p = subprocess.run(args, cwd=cwd, env=dict(env, **extra), stdout=log, stderr=subprocess.STDOUT)
    print(f'{name}: exit={p.returncode}, seconds={time.monotonic()-start:.1f}', flush=True)
    if p.returncode:
        print('\n'.join((logs/(name+'.log')).read_text().splitlines()[-45:]), flush=True)
        sys.exit(p.returncode)
