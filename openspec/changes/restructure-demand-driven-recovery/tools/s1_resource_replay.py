#!/usr/bin/env python3
"""Interleaved S1 method/class/query replay using the existing S0 measurement runner."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys

DEFAULT_CHANGE = Path(__file__).resolve().parents[1]
DEFAULT_OBS = DEFAULT_CHANGE / 'results/s0-observed-v3'


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')


def load_module(path):
    spec = importlib.util.spec_from_file_location('s0_campaign_reused', path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def pair_metrics(row):
    """Keep the requested counters in a compact, repeat-level comparison record."""
    if row.get('exit_code') != 0 or len(row.get('samples', [])) != 1:
        return None
    wrapped = row['samples'][0]
    sample = wrapped['sample']
    java = sample['retention']['java_end']
    reader = sample['retention']['reader_end']
    return {
        'open_actual_reads': wrapped.get('open_actual_reads'),
        'request_actual_reads': {
            k: sample['observed_work']['reader'].get(k)
            for k in ('read_attempts', 'read_bytes', 'read_eofs', 'read_errors', 'file_read_attempts', 'file_read_bytes', 'file_read_eofs', 'file_read_errors')
        },
        'body': {
            'd0_body_decodes': sample['counts'].get('body_decodes'),
            'reader_body_decode_attempts': sample['observed_work']['reader'].get('body_decode_attempts'),
        },
        'ir': {k: sample['usage'].get(k) for k in ('ir_items', 'ir_edges')},
        'ast': {k: java.get(k) for k in ('expr_peak', 'stmt_peak', 'method_ast_sources_peak')},
        'candidate_peak': java.get('class_candidate_payloads_peak'),
        'time': {
            'open_micros': wrapped.get('open_micros'),
            'operation_micros': sample.get('micros'),
            'wall_seconds': row.get('wall_seconds'),
        },
        'rss_process_peak_bytes': row.get('rss_process_peak_bytes'),
        'sampled_process_group_rss_peak_bytes': row.get('sampled_process_group_rss_peak_bytes'),
        'usage': sample.get('usage'),
        'semantic_text_digest': sample.get('semantic_text_digest'),
        'retention_reader_peaks': {k: v for k, v in reader.items() if k.endswith('_peak')},
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--new-harness', required=True, type=Path)
    ap.add_argument('--out-dir', required=True, type=Path)
    ap.add_argument('--old-harness', type=Path)
    ap.add_argument('--change-dir', type=Path, default=DEFAULT_CHANGE)
    args = ap.parse_args()

    change = args.change_dir.resolve()
    obs = change / 'results/s0-observed-v3'
    source_plan = json.loads((obs / 'resource-plan.json').read_text())
    source_raw = json.loads((obs / 'resource-raw.json').read_text())
    source_summary = json.loads((obs / 'resource-summary.json').read_text())
    binary_meta = json.loads((obs / 'resource-binary.json').read_text())
    old_harness = (args.old_harness or Path(binary_meta['path'])).resolve()
    new_harness = args.new_harness.resolve()
    out = args.out_dir.resolve()

    configs = [c for c in source_plan['configs']
               if c['shape'] in {'single', 'class', 'query'}]
    ids = {c['id'] for c in configs}
    baseline_rows = [r for r in source_raw if r['config_id'] in ids]
    baseline = {c['id']: [r for r in baseline_rows if r['config_id'] == c['id']] for c in configs}
    if len(configs) != 30 or any(len(baseline[c['id']]) != source_plan['repeats'] for c in configs):
        raise SystemExit('S0 baseline is not the expected 30 x 10 complete method/class/query matrix')
    baseline_summary = {x['id']: x for x in source_summary['configs']}
    if any(not baseline_summary[c['id']]['complete'] for c in configs):
        raise SystemExit('S0 baseline summary marks a selected S1 config incomplete')
    if not old_harness.is_file() or not new_harness.is_file():
        raise SystemExit('old/new harness path is not a file; no run was started')
    if out.exists() and any(out.iterdir()):
        raise SystemExit('output directory is nonempty; choose a fresh output directory')

    campaign = load_module(change / 'tools/s0_campaign.py')
    for config in configs:
        assert campaign.pin(Path(config['input'])) == config['input_pin'], 'frozen artifact changed'
    campaign.ROOT = change.parents[2]
    # Copy the exact pinned request configs, repeats, thresholds and harness guard policy.
    replay_plan = {k: v for k, v in source_plan.items() if k != 'configs'}
    replay_plan.update(configs=configs, source_plan=campaign.pin(obs / 'resource-plan.json'),
                       source_summary=campaign.pin(obs / 'resource-summary.json'),
                       source_raw=campaign.pin(obs / 'resource-raw.json'),
                       baseline_source='s0-observed-v3',
                       ordering='10 rounds; rotate configuration start as S0; each configuration runs old/new adjacent, with pair order alternating by round and rotated config index; fresh process for every invocation')
    write(out / 'resource-plan.json', replay_plan)
    write(out / 'baseline-source-pins.json', {
        'plan': campaign.pin(obs / 'resource-plan.json'),
        'raw': campaign.pin(obs / 'resource-raw.json'),
        'summary': campaign.pin(obs / 'resource-summary.json'),
        'old_harness': campaign.pin(old_harness),
        'new_harness': campaign.pin(new_harness),
    })

    rows = {'old': [], 'new': []}
    for side in rows:
        (out / side).mkdir(parents=True, exist_ok=True)
        write(out / side / 'resource-plan.json', replay_plan)
        write(out / side / 'resource-raw.json', [])
    n = source_plan['repeats']
    for repeat in range(n):
        for index in range(len(configs)):
            config = configs[(repeat + index) % len(configs)]
            order = ('old', 'new') if (repeat + index) % 2 == 0 else ('new', 'old')
            for side in order:
                campaign.OUT = out / side
                campaign.BINARY = old_harness if side == 'old' else new_harness
                row = campaign.run_one(config, repeat)
                rows[side].append(row)
                write(out / side / 'resource-raw.json', rows[side])
                print(repeat, config['id'], side, row['exit_code'], row['guard'], flush=True)
        print('round-complete', repeat + 1, flush=True)

    summaries = {}
    for side in ('old', 'new'):
        campaign.OUT = out / side
        campaign.summarize()
        summaries[side] = json.loads((out / side / 'resource-summary.json').read_text())

    by_side = {side: {r['config_id'] + ':' + str(r['repeat']): r for r in rows[side]} for side in rows}
    comparisons = []
    for config in configs:
        entries = []
        for repeat in range(n):
            key = config['id'] + ':' + str(repeat)
            entries.append({
                'repeat': repeat,
                'old': pair_metrics(by_side['old'][key]),
                'new': pair_metrics(by_side['new'][key]),
            })
        old_summary = next(x for x in summaries['old']['configs'] if x['id'] == config['id'])
        new_summary = next(x for x in summaries['new']['configs'] if x['id'] == config['id'])
        comparisons.append({
            'config': config,
            'old_summary': old_summary,
            'new_summary': new_summary,
            'paired_repeats': entries,
            'candidate_peak_change': 'report raw and summary values; any increase requires a semantic explanation and is not treated as a gate pass',
        })
    write(out / 'resource-comparison.json', {
        'schema': 'jarde-s1-resource-comparison-v1',
        'source_plan': campaign.pin(obs / 'resource-plan.json'),
        'thresholds': source_plan['thresholds'],
        'configs': comparisons,
        'gate_policy': 'No thresholds relaxed or inferred here. Apply the unchanged S0 formulas to each side; report deltas and semantic output digests. Candidate-count growth is explicitly reviewed, never auto-approved.',
    })


if __name__ == '__main__':
    main()
