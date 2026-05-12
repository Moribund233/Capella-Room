#!/usr/bin/env python3
"""
Capella Room 项目重命名脚本
将项目中所有 'seredeli' (不区分大小写) 的变体智能替换为 'capella'。

替换规则:
  - seredeli  → capella   (全小写)
  - Seredeli  → Capella   (首字母大写)
  - SEREDELI  → CAPELLA   (全大写)
  - SereDeli  → Capella   (混合大小写)

排除规则: 以 gitignore 为准（通过 git check-ignore 判断）
项目根目录名称不受影响。
"""

import os
import re
import subprocess
import sys
import argparse
from pathlib import Path


def is_git_ignored(file_path: Path, root_dir: Path) -> bool:
    """使用 git check-ignore 判断文件是否被 git 忽略"""
    try:
        result = subprocess.run(
            ["git", "check-ignore", "-q", str(file_path)],
            cwd=str(root_dir),
            capture_output=True,
            text=True,
        )
        return result.returncode == 0
    except FileNotFoundError:
        return False


def is_binary(file_path: Path) -> bool:
    """快速判断文件是否为二进制"""
    try:
        with open(file_path, "rb") as f:
            chunk = f.read(8192)
        return b"\0" in chunk
    except Exception:
        return True


def get_case_style(text: str) -> str:
    """根据匹配文本的大小写风格返回对应的替换文本"""
    if text.isupper():
        return "CAPELLA"
    elif text[0].isupper():
        return "Capella"
    else:
        return "capella"


# 通过构建避免脚本自身上出现字面量 "seredeli" 被自身替换
_OLD_PARTS = ["ser", "edeli"]


def _old_pattern() -> re.Pattern:
    return re.compile(r"(?i)" + "".join(_OLD_PARTS))


def _skip_self(rel_path: Path) -> bool:
    """跳过本脚本自身"""
    return rel_path == Path("script") / "python" / "rename_to_capella.py"


def collect_target_files(root_dir: Path) -> list[Path]:
    """收集所有需要处理的文件（排除 gitignore 和二进制）"""
    pattern = _old_pattern()
    files = []

    for dirpath, dirnames, filenames in os.walk(root_dir):
        # 跳过 .git 目录
        if ".git" in dirpath.split(os.sep):
            dirnames[:] = []
            continue

        for filename in filenames:
            file_path = Path(dirpath) / filename
            rel_path = file_path.relative_to(root_dir)

            # 跳过本脚本自身
            if _skip_self(rel_path):
                continue

            # 跳过 node_modules 等常见大目录以提高性能
            if any(p in rel_path.parts for p in (".git", "node_modules", "target")):
                continue

            # 跳过 gitignore 规则的文件
            if is_git_ignored(file_path, root_dir):
                continue

            # 跳过二进制文件
            if is_binary(file_path):
                continue

            # 检查是否包含旧名称
            try:
                with open(file_path, "r", encoding="utf-8", errors="replace") as f:
                    content = f.read()
            except Exception:
                continue

            if pattern.search(content):
                files.append(file_path)

    return files


def run(root_dir: Path, dry_run: bool = False) -> tuple[int, int]:
    """
    执行重命名替换。
    返回 (修改文件数, 总替换处数)
    """
    pattern = _old_pattern()
    files = collect_target_files(root_dir)

    if not files:
        print("未找到包含旧名称的文件。")
        return 0, 0

    modified_files = 0
    total_replacements = 0

    for file_path in files:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()

        def replace_func(m):
            return get_case_style(m.group(0))

        new_content, count = pattern.subn(replace_func, content)

        if count > 0:
            rel_path = file_path.relative_to(root_dir)
            total_replacements += count

            if dry_run:
                print(f"  [DRY-RUN]  {rel_path}  ({count} 处)")
            else:
                with open(file_path, "w", encoding="utf-8") as f:
                    f.write(new_content)
                print(f"  [MODIFY]  {rel_path}  ({count} 处)")

            modified_files += 1

    return modified_files, total_replacements


def main():
    parser = argparse.ArgumentParser(
        description="将项目中 'seredeli'(不区分大小写) 替换为 'capella'(保持原大小写风格)"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="预览模式，不实际修改文件",
    )
    args = parser.parse_args()

    # 脚本位于 project/script/python/，上溯三级得项目根目录
    root_dir = Path(__file__).resolve().parent.parent.parent
    os.chdir(root_dir)

    action_label = "[预览]" if args.dry_run else ""
    print(f"{action_label} 项目根目录: {root_dir}")
    print(f"{action_label} 模式: {'预览 (不会修改)' if args.dry_run else '执行'}")
    print("=" * 60)

    modified, total = run(root_dir, dry_run=args.dry_run)

    print("=" * 60)
    print(f"{action_label} 完成! 找到 {modified} 个文件, 共 {total} 处替换")

    return 0


if __name__ == "__main__":
    sys.exit(main())
