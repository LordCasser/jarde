"""Race-tolerant regular-file byte scan for a Cargo target directory.

This module deliberately has no process or policy logic. Call it from the existing
runner's target_bytes() hook; leave that runner's thresholds and process-group guard
unchanged.
"""

from pathlib import Path
import stat


def regular_file_bytes(root: Path) -> int:
    """Sum one stat snapshot per discovered regular-file path below root.

    A path concurrently removed before stat contributes zero. A path removed after
    a successful stat contributes that one observed size until the next scan. Other
    filesystem errors propagate so the caller cannot mistake an incomplete scan for
    a safe target size.
    """
    total = 0
    for path in root.rglob("*"):
        try:
            info = path.stat()
        except FileNotFoundError:
            continue
        if stat.S_ISREG(info.st_mode):
            total += info.st_size
    return total
