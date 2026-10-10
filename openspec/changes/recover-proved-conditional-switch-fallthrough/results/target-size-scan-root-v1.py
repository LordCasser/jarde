"""Measure changing Cargo output without an is_file/stat race."""
from pathlib import Path
import stat


def target_bytes(root: Path) -> int:
    total = 0
    for path in (root / 'target').rglob('*'):
        try:
            info = path.stat()
        except FileNotFoundError:
            # Cargo may remove or atomically replace an entry during this scan.
            continue
        if stat.S_ISREG(info.st_mode):
            total += info.st_size
    return total
