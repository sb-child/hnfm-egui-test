#!/usr/bin/env python3
"""下载 Material Symbols 图标字体，并生成 src/icons/generated.rs。

字体和名字表来自同一个固定 commit，避免字体更新后名字表漂移。
生成的名字表只包含图标名，运行时由 `Icon` 常量暴露，调用方不接触字符串。
"""

import shutil
import subprocess
import sys
from pathlib import Path

import requests

# google/material-design-icons 的固定 commit。
COMMIT = "10ecc14790612ca8fd164e3f4ffe9c16fc50d5b7"
BASE_URL = f"https://raw.githubusercontent.com/google/material-design-icons/{COMMIT}"

FONT_NAME = "MaterialSymbolsRounded"
FONT_URL = (
    f"{BASE_URL}/variablefont/MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.ttf"
)
CODEPOINTS_URL = (
    f"{BASE_URL}/variablefont/MaterialSymbolsRounded%5BFILL%2CGRAD%2Copsz%2Cwght%5D.codepoints"
)
LICENSE_URL = f"{BASE_URL}/LICENSE"

ICONS_DIR = Path(__file__).resolve().parent / "src" / "icons"
CHUNK_SIZE = 128 * 1024

# `Icon` 上已有的关联常量，生成的名字不能和它们重名。
RESERVED_NAMES = {"ALL"}


def download(url: str, dest: Path, label: str) -> None:
    """下载到目标路径，带进度显示。"""
    print(f"  下载中 {url}")
    # 关掉压缩，content-length 才和实际字节数一致，进度不会超过 100%。
    resp = requests.get(
        url, stream=True, timeout=60, headers={"Accept-Encoding": "identity"}
    )
    resp.raise_for_status()
    total = int(resp.headers.get("content-length", 0))
    data = bytearray()
    for chunk in resp.iter_content(chunk_size=CHUNK_SIZE):
        data.extend(chunk)
        if total > 0:
            pct = len(data) / total * 100
            sys.stdout.write(f"\r  {label}  {len(data)}/{total} ({pct:.0f}%)")
            sys.stdout.flush()
    print(f"\r  {label}  {len(data)} bytes  ok")
    dest.write_bytes(data)


def const_name(icon_name: str) -> str:
    """图标名转 Rust 常量名。数字开头的加 N 前缀，因为标识符不能以数字开头。"""
    upper = icon_name.upper()
    if upper[0].isdigit():
        upper = "N" + upper
    return upper


def parse_codepoints(text: str) -> list[str]:
    """解析 `<name> <codepoint>` 每行一条的文件。"""
    names: list[str] = []
    for line in text.splitlines():
        parts = line.split()
        if not parts:
            continue
        if len(parts) != 2:
            raise ValueError(f"无法解析 codepoints 行: {line!r}")
        names.append(parts[0])
    return names


def check_names(names: list[str]) -> None:
    """检查重复、非法字符和保留名，问题都在生成前暴露。"""
    seen: dict[str, str] = {}
    for name in names:
        if not name[0].isalnum() and name[0] != "_":
            raise ValueError(f"图标名不以字母或下划线开头: {name!r}")
        for ch in name:
            if not (ch.isascii() and (ch.isalnum() or ch == "_")):
                raise ValueError(f"图标名包含非法字符: {name!r}")
        const = const_name(name)
        if const in RESERVED_NAMES:
            raise ValueError(f"图标名 {name!r} 生成 {const}，与已有常量冲突")
        if const in seen:
            raise ValueError(f"图标名 {name!r} 和 {seen[const]!r} 生成同一个常量 {const}")
        seen[const] = name
    if len(set(names)) != len(names):
        raise ValueError("codepoints 中存在重复的图标名")


def render_module(names: list[str]) -> str:
    """生成 Rust 源文件。names 必须已按字典序排序。"""
    lines: list[str] = [
        "//! Material Symbols 图标名表。",
        "//!",
        "//! 由 `download_icons.py` 生成，不要手动修改。",
        f"//! 字体: google/material-design-icons@{COMMIT} 的 {FONT_NAME}。",
        "",
        "use super::Icon;",
        "",
        "impl Icon {",
    ]
    for name in names:
        lines.append(f'    pub const {const_name(name)}: Self = Self("{name}");')
    lines.extend(
        [
            "}",
            "",
            "/// 按字典序排列的全部图标名，`Icon::from_name` 依赖这个顺序做二分查找。",
            "pub(crate) const ICON_NAMES: &[&str] = &[",
        ]
    )
    for name in names:
        lines.append(f'    "{name}",')
    lines.extend(["];", ""])
    return "\n".join(lines)


def main() -> None:
    ICONS_DIR.mkdir(parents=True, exist_ok=True)

    download(FONT_URL, ICONS_DIR / f"{FONT_NAME}.ttf", f"{FONT_NAME}.ttf")
    download(LICENSE_URL, ICONS_DIR / "LICENSE", "LICENSE")

    print("\n[图标名]")
    print(f"  下载中 {CODEPOINTS_URL}")
    resp = requests.get(CODEPOINTS_URL, timeout=60)
    resp.raise_for_status()
    names = parse_codepoints(resp.text)
    if not names:
        raise ValueError("codepoints 文件为空")
    check_names(names)
    # 文件顺序不保证，二分查找要求按名字排序。
    names.sort()
    print(f"  {len(names)} 个图标")

    dest = ICONS_DIR / "generated.rs"
    dest.write_text(render_module(names), encoding="utf-8")
    # 生成的长行交给 rustfmt，保持和 `cargo fmt --check` 一致。
    if shutil.which("rustfmt"):
        subprocess.run(["rustfmt", "--edition", "2024", str(dest)], check=False)
    print(f"  写入 {dest}")


if __name__ == "__main__":
    try:
        main()
    except (requests.RequestException, ValueError) as e:
        print(f"\n错误: {e}")
        sys.exit(1)
