import hashlib, importlib.util, json, re, tarfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RUN = Path('/private/tmp/jarde-seq1-focused-root-v6-20261018')
OUT = Path('/private/tmp/jarde-seq1-focused-review-root-v2-20261018')
def pin(path):
    path = Path(path)
    data = path.read_bytes()
    return {'path': str(path), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}

d = json.loads((RUN / 'execution.json').read_bytes())
assert d['status'] == 'commands_passed_pending_independent_acceptance'
assert d['default_stack'] and d['source_unchanged'] and d['source_changes_accepted']
assert d['source_before'] == d['source_after'] and d['target_absent']
assert not (ROOT / 'target').exists()
for item in d['source_after']:
    assert pin(item['path']) == item
names, summaries = [], []
assert len(d['commands']) == 6
for c in d['commands']:
    assert c['exit_code'] == 0 and c.get('guard_stop') is None
    assert c.get('peak_target_bytes', 0) <= 1024**3
    assert c.get('free_bytes_after', 5*1024**3) >= 5*1024**3
    assert 'RUST_MIN_STACK' not in c.get('env_overrides', {})
    for stream in c['streams'].values():
        assert pin(stream['path']) == stream
    text = Path(c['streams']['stdout']['path']).read_text()
    current = re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ok$', text, re.M)
    parsed = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', text)
    if parsed:
        assert len(parsed) == 1
        passed, failed, ignored = map(int, parsed[0])
        assert failed == ignored == 0 and len(current) == passed
        summaries.append(passed)
        names.extend(current)
assert summaries == [486, 4, 181] and len(names) == 671
critical = [n for n in names if 'statementful_loop_header_tests::' in n or 'field_test_copy_tests::' in n]
public = ['effectful_sequential_loop_headers_preserve_complete_supplemental_methods',
          'original_sequential_loops_keep_both_statementful_headers_and_full_classes',
          'saved_field_write_copies_cover_both_compilers_and_debug_profiles',
          'statementful_loop_late_budget_cancellation_and_deadline_publish_no_partial_result']
assert len(critical) == 7
for name in critical + public:
    assert names.count(name) == 1
assert d['commands'][-1]['argv'] == ['cargo', 'clean']
assert pin(RUN / 'candidate-cli') == d['cli']
assert d['cli']['sha256'] == '0bb89f827c1e2b6b0e215d1940299be046a7adc57866389d87e6595ba0295677'
archive = ROOT / 'openspec/changes/recover-statementful-loop-headers/results/focused-root-v6/capture.tar.gz'
spec = importlib.util.spec_from_file_location('audit', '/private/tmp/jarde-prefixed-retry-all-archives-luna-v1-20261018/audit_archives.py')
audit = importlib.util.module_from_spec(spec); spec.loader.exec_module(audit)
review = audit.audit_one(archive)
assert review['tar_member_count'] == 21
with tarfile.open(archive) as tf:
    found = [m for m in tf.getmembers() if m.name.endswith('/execution.json') or m.name == 'execution.json']
    assert len(found) == 1 and tf.extractfile(found[0]).read() == (RUN / 'execution.json').read_bytes()
OUT.mkdir()
result = {'status': 'accepted', 'execution': pin(RUN / 'execution.json'), 'cli': d['cli'],
          'test_summaries': summaries, 'passing_appearances': len(names), 'critical_tests': critical + public,
          'source_count': len(d['source_after']), 'source_unchanged': True, 'target_absent': True,
          'archive': pin(archive), 'archive_members': 21,
          'checker_history': ['Earlier inline checker omitted the standard should-panic suffix, counted 670 names, and failed before writing acceptance. A subsequent inspection used c[test_summary_check] on cleanup and failed with KeyError. Neither was a product test failure; this reviewer checks all 671 names and raw bytes.']}
(OUT / 'review.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'status': 'accepted', 'tests': len(names), 'review': str(OUT / 'review.json')}))
