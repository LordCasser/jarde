"""Change the sole iinc operand from slot 0 to initialized parameter slot 1."""

from pathlib import Path

class_file = Path(__file__).parent / "v8/DifferentSlotArrayElement.class"
data = bytearray(class_file.read_bytes())
needle = b"\x84\x00\x01"
assert data.count(needle) == 1
at = data.index(needle)
data[at + 1] = 1
class_file.write_bytes(data)
