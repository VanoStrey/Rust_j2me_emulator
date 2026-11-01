#!/usr/bin/env python3
import os, sys, json, time, subprocess
from concurrent.futures import ProcessPoolExecutor

exe = os.path.abspath('target/release/j2me-cli')
tmp_dir = os.path.abspath('experiments/market_tmp')
os.makedirs(tmp_dir, exist_ok=True)

def run_one(item):
    idx, jar_name, jar_path = item
    tmp_json = os.path.join(tmp_dir, f'test_{idx}.json')
    cmd = [exe, '--run-single', jar_path, '--ticks', '300', '--output-json', tmp_json]
    t0 = time.time()
    try:
        proc = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)
        elapsed = time.time() - t0
        if os.path.exists(tmp_json):
            with open(tmp_json, 'r') as f:
                res = json.load(f)
            try:
                os.remove(tmp_json)
            except:
                pass
            return {
                'index': idx,
                'jar_name': jar_name,
                'jar_path': jar_path,
                'runtime_status': res.get('runtime_status', 'UNKNOWN'),
                'exit_reason': res.get('exit_reason', ''),
                'last_error': res.get('last_error', ''),
                'exception_class': res.get('last_exception_class'),
                'exception_message': res.get('last_exception_message'),
                'exception_location': res.get('last_exception_location'),
                'paint_calls': res.get('paint_calls', 0),
                'instructions': res.get('executed_instructions', 0),
                'active_thread_stacks': res.get('active_thread_stacks', []),
                'elapsed': round(elapsed, 3)
            }
        else:
            return {
                'index': idx,
                'jar_name': jar_name,
                'jar_path': jar_path,
                'runtime_status': 'NO_OUTPUT',
                'exit_reason': 'Process exited without output',
                'last_error': 'No output json produced',
                'exception_class': None,
                'exception_message': None,
                'exception_location': None,
                'paint_calls': 0,
                'instructions': 0,
                'active_thread_stacks': [],
                'elapsed': round(elapsed, 3)
            }
    except subprocess.TimeoutExpired:
        return {
            'index': idx,
            'jar_name': jar_name,
            'jar_path': jar_path,
            'runtime_status': 'WALL_TIMEOUT',
            'exit_reason': 'Process killed by 10s wall timeout',
            'last_error': 'Timeout',
            'exception_class': None,
            'exception_message': None,
            'exception_location': None,
            'paint_calls': 0,
            'instructions': 0,
            'active_thread_stacks': [],
            'elapsed': 10.0
        }

def main():
    with open('experiments/market_tmp_222_list.json') as f:
        items = json.load(f)

    tasks = [(i, item['jar_name'], item['jar_path']) for i, item in enumerate(items)]
    print(f'Starting benchmark on {len(tasks)} games...')
    results = []
    with ProcessPoolExecutor(max_workers=8) as pool:
        for res in pool.map(run_one, tasks):
            results.append(res)

    with open('experiments/phase510_benchmark_222.json', 'w') as f:
        json.dump(results, f, indent=2)

    counts = {}
    for r in results:
        s = r['runtime_status']
        counts[s] = counts.get(s, 0) + 1
    print('Current 222 counts:', counts)

    guest_excs = [r for r in results if r['runtime_status'] == 'GUEST_EXCEPTION']
    print(f'GUEST_EXCEPTION count: {len(guest_excs)}')
    for g in guest_excs:
        print(f"- {g['jar_name']}: {g['exception_class']} ({g['exception_message']}) at {g['exception_location']}")

if __name__ == '__main__':
    main()
