#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
RE-Framework index 合并器（core.artifact 配套工具，spec §5.1）

用途: 并行 worker 各自交付 .artifacts/**/index-entry.yaml 片段后，
      合并到根 .artifacts/index.yaml，替代主会话手动合并（实战项目实证：
      5 个并行 worker 片段手动合并，biome-fix 5 条差点漏合并）。

用法:
    python scripts/merge_index.py <project_dir> [--dry-run] [--verbose]

行为:
    - 扫描 <project>/.artifacts/**/index-entry.yaml（递归）
    - 按 id 合并 entries 到 <project>/.artifacts/index.yaml（无根文件则初始化）
    - 冲突检测（不静默覆盖）:
        * id 重复但 path 不同           → CONFLICT（跳过）
        * id/path 相同但 status 不同     → CONFLICT（跳过，人工裁决）
        * id/path/status 相同           → 已存在，跳过（幂等）
    - --dry-run 只报告不写盘

退出码: 0 = 合并完成（可能含冲突警告）；2 = 参数/IO 错误
"""
import argparse
import glob
import os
import sys

try:
    import yaml
except ImportError:
    print('需要 PyYAML: pip install pyyaml', file=sys.stderr)
    sys.exit(2)

INDEX_ENTRY_NAME = 'index-entry.yaml'
ROOT_INDEX_NAME = 'index.yaml'
ENTRY_KEYS = ('id', 'path', 'kind', 'status')


def load_yaml(path):
    try:
        with open(path, encoding='utf-8') as f:
            return yaml.safe_load(f) or {}
    except Exception as e:
        print(f'! 解析失败 {path}: {e}', file=sys.stderr)
        return None


def norm_entries(data):
    """归一化 entries 为 (id, path, kind, status) 列表，缺字段补空"""
    out = []
    for e in data.get('entries', []) or []:
        if not isinstance(e, dict):
            continue
        out.append({k: e.get(k, '') for k in ENTRY_KEYS})
    return out


def collect_fragments(artifacts_dir):
    return sorted(glob.glob(os.path.join(artifacts_dir, '**', INDEX_ENTRY_NAME), recursive=True))


def main():
    ap = argparse.ArgumentParser(description='合并 .artifacts/**/index-entry.yaml 到根 index.yaml')
    ap.add_argument('project_dir', help='项目目录（含 .artifacts/）')
    ap.add_argument('--dry-run', action='store_true', help='只报告不写盘')
    ap.add_argument('--verbose', action='store_true', help='详细输出每个片段')
    args = ap.parse_args()

    artifacts = os.path.join(args.project_dir, '.artifacts')
    if not os.path.isdir(artifacts):
        print(f'! 找不到 .artifacts/: {artifacts}', file=sys.stderr)
        sys.exit(2)

    root_path = os.path.join(artifacts, ROOT_INDEX_NAME)
    root_entries = []
    if os.path.isfile(root_path):
        root = load_yaml(root_path)
        if root is None:
            sys.exit(2)
        root_entries = norm_entries(root)

    fragments = collect_fragments(artifacts)
    if not fragments:
        print(f'未发现 index-entry.yaml 片段（.artifacts/**/{INDEX_ENTRY_NAME}）')
        sys.exit(0)

    by_id = {e['id']: e for e in root_entries if e['id']}
    conflicts, added, skipped, sources = [], 0, 0, {}

    for frag in fragments:
        data = load_yaml(frag)
        if data is None:
            continue
        frag_entries = norm_entries(data)
        if args.verbose:
            print(f'[片段] {os.path.relpath(frag, artifacts)}: {len(frag_entries)} entries')
        for e in frag_entries:
            if not e['id']:
                print(f'! 空 id: {frag}', file=sys.stderr)
                continue
            existing = by_id.get(e['id'])
            if existing is None:
                by_id[e['id']] = e
                sources[e['id']] = frag
                added += 1
            elif (existing['path'] == e['path'] and existing['status'] == e['status']):
                skipped += 1  # 幂等：已存在且一致
            else:
                conflicts.append({
                    'id': e['id'],
                    'existing': existing,
                    'incoming': e,
                    'fragment': frag,
                })

    # 报告
    print(f'=== index 合并: {len(fragments)} 片段, +{added} 新增, {skipped} 已存在, {len(conflicts)} 冲突 ===')
    for c in conflicts:
        print(f'CONFLICT: {c["id"]}')
        print(f'  已有: {c["existing"].get("path")} [{c["existing"].get("status")}]')
        print(f'  新到: {c["incoming"].get("path")} [{c["incoming"].get("status")}] @ {os.path.relpath(c["fragment"], artifacts)}')
        print('  → 未合并，人工裁决（路径不同或状态不一致）')

    if args.dry_run:
        print('(dry-run) 未写盘')
        sys.exit(0)

    # 写回根 index（保留原 schema 字段 + 更新 entries）
    existing_root = load_yaml(root_path) if os.path.isfile(root_path) else {}
    merged = {
        'schema_version': 1,
        'project': existing_root.get('project', os.path.basename(args.project_dir)),
        'module': 'core',
        'entries': sorted(by_id.values(), key=lambda e: e['id']),
    }
    with open(root_path, 'w', encoding='utf-8') as f:
        yaml.safe_dump(merged, f, allow_unicode=True, sort_keys=False)
    print(f'已写回 {os.path.relpath(root_path, args.project_dir)}（{len(merged["entries"])} entries）')
    sys.exit(0 if not conflicts else 0)  # 冲突仅报告，不阻塞（人工裁决）


if __name__ == '__main__':
    main()
