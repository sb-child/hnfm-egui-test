#!/usr/bin/env python3
"""下载项目的 3 个字体文件到 src/fonts/"""

import io
import sys
import zipfile
from pathlib import Path

import requests

FONTS: list[dict] = [
    {
        "name": "SourceHanSansSC-VF.otf",
        "url": "https://raw.githubusercontent.com/adobe-fonts/source-han-sans/release/Variable/OTF/SourceHanSansSC-VF.otf",
        "zip_member": None,
    },
    {
        "name": "SourceHanSerifSC-VF.otf",
        "url": "https://raw.githubusercontent.com/adobe-fonts/source-han-serif/release/Variable/OTF/SourceHanSerifSC-VF.otf",
        "zip_member": None,
    },
    {
        "name": "MapleMonoNL-NF-CN-Regular-unhinted.ttf",
        "url": "https://github.com/subframe7536/maple-font/releases/download/v7.9/MapleMonoNL-NF-CN-unhinted.zip",
        "zip_member": "MapleMonoNL-NF-CN-Regular.ttf",
    },
]

FONTS_DIR = Path(__file__).resolve().parent / "src" / "fonts"
CHUNK_SIZE = 128 * 1024


def download_direct(name: str, url: str, dest: Path) -> None:
    """直接从 URL 下载字体文件，带进度显示"""
    print(f"  下载中 {url}")
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    total = int(resp.headers.get("content-length", 0))
    downloaded = 0
    with open(dest, "wb") as f:
        for chunk in resp.iter_content(chunk_size=CHUNK_SIZE):
            f.write(chunk)
            downloaded += len(chunk)
            if total > 0:
                pct = downloaded / total * 100
                sys.stdout.write(f"\r  {name}  {downloaded}/{total} ({pct:.0f}%)")
                sys.stdout.flush()
    print(f"\r  {name}  {downloaded} bytes  ✓")


def download_and_extract(name: str, url: str, dest: Path, zip_member: str) -> None:
    """下载 zip 并提取指定的内部文件"""
    print(f"  下载中 {url}")
    resp = requests.get(url, stream=True)
    resp.raise_for_status()
    total = int(resp.headers.get("content-length", 0))
    downloaded = 0
    data = bytearray()
    for chunk in resp.iter_content(chunk_size=CHUNK_SIZE):
        data.extend(chunk)
        downloaded += len(chunk)
        if total > 0:
            pct = downloaded / total * 100
            sys.stdout.write(f"\r  {url.rsplit('/', 1)[-1]}  {downloaded}/{total} ({pct:.0f}%)")
            sys.stdout.flush()
    print()

    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        if zip_member not in zf.namelist():
            available = [n for n in zf.namelist() if n.endswith(".ttf")]
            print(f"  错误: 在 zip 中未找到 {zip_member}")
            if available:
                print(f"  可用的 ttf 文件: {available}")
            sys.exit(1)
        print(f"  解压 {zip_member} -> {dest}")
        dest.write_bytes(zf.read(zip_member))
    print(f"  {name}  ✓")


def main() -> None:
    FONTS_DIR.mkdir(parents=True, exist_ok=True)

    for font in FONTS:
        name: str = font["name"]
        url: str = font["url"]
        zip_member: str | None = font["zip_member"]
        dest = FONTS_DIR / name
        print(f"\n[{name}]")
        try:
            if zip_member:
                download_and_extract(name, url, dest, zip_member)
            else:
                download_direct(name, url, dest)
        except requests.RequestException as e:
            print(f"  下载失败: {e}")
            sys.exit(1)

    print(f"\n全部 {len(FONTS)} 个字体下载完成 -> {FONTS_DIR}")


if __name__ == "__main__":
    main()
