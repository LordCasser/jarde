"""Freeze verifier-valid TWR headers with an extra effect or a mismatched Store value."""
from pathlib import Path
import hashlib
import difflib
import shutil
import subprocess
import struct
import tempfile

BASE = Path(__file__).resolve().parent
SOURCE = BASE / "release8" / "MultiResourceTwr.class"
OUT = BASE / "effectful-header-negative"
OUT.mkdir(exist_ok=True)
TARGET = OUT / "MultiResourceTwr.class"
shutil.copy2(SOURCE, TARGET)
data = bytearray(TARGET.read_bytes())


def u2(pos):
    return struct.unpack_from(">H", data, pos)[0]


def u4(pos):
    return struct.unpack_from(">I", data, pos)[0]


def put_u2(pos, value):
    struct.pack_into(">H", data, pos, value)


def put_u4(pos, value):
    struct.pack_into(">I", data, pos, value)


def cp_utf8():
    pos = 8
    count = u2(pos)
    pos += 2
    values = {}
    index = 1
    while index < count:
        tag = data[pos]
        pos += 1
        if tag == 1:
            size = u2(pos)
            pos += 2
            values[index] = bytes(data[pos:pos + size]).decode("utf-8", "replace")
            pos += size
        elif tag in (3, 4):
            pos += 4
        elif tag in (5, 6):
            pos += 8
            index += 1
        elif tag in (7, 8, 16, 19, 20):
            pos += 2
        elif tag in (9, 10, 11, 12, 17, 18):
            pos += 4
        elif tag == 15:
            pos += 3
        else:
            raise ValueError(f"unknown constant-pool tag {tag}")
        index += 1
    return values, pos


def shifted_offset(value, threshold=9, amount=3):
    return value + amount if value >= threshold else value


def rewrite_stack_map(payload, threshold=9, amount=3):
    """Adjust frame BCIs while preserving each frame's locals and stack payload."""
    pos = 0
    count = struct.unpack_from(">H", payload, pos)[0]
    pos += 2
    frames = []
    old_previous = -1
    for _ in range(count):
        frame_type = payload[pos]
        pos += 1
        if frame_type <= 63:
            delta = frame_type
            kind, body = "same", b""
        elif frame_type <= 127:
            delta = frame_type - 64
            start = pos
            pos = skip_verification_type(payload, pos)
            kind, body = "same_locals", payload[start:pos]
        elif frame_type == 247:
            delta = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            start = pos
            pos = skip_verification_type(payload, pos)
            kind, body = "same_locals", payload[start:pos]
        elif 248 <= frame_type <= 250:
            delta = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            kind, body = "chop", bytes([251 - frame_type])
        elif frame_type == 251:
            delta = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            kind, body = "same_extended", b""
        elif 252 <= frame_type <= 254:
            delta = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            count_locals = frame_type - 251
            start = pos
            for _ in range(count_locals):
                pos = skip_verification_type(payload, pos)
            kind, body = "append", payload[start:pos]
        elif frame_type == 255:
            delta = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            start = pos
            locals_count = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            for _ in range(locals_count):
                pos = skip_verification_type(payload, pos)
            stack_count = struct.unpack_from(">H", payload, pos)[0]
            pos += 2
            for _ in range(stack_count):
                pos = skip_verification_type(payload, pos)
            kind, body = "full", payload[start:pos]
        else:
            raise ValueError(f"reserved StackMapTable frame type {frame_type}")
        old_absolute = old_previous + delta + 1
        new_absolute = shifted_offset(old_absolute, threshold, amount)
        frames.append((kind, body, new_absolute))
        old_previous = old_absolute
    if pos != len(payload):
        raise ValueError("unexpected trailing StackMapTable bytes")

    out = bytearray(struct.pack(">H", count))
    previous = -1
    for kind, body, absolute in frames:
        delta = absolute - previous - 1
        if kind == "same":
            if delta <= 63:
                out.append(delta)
            else:
                out.append(251)
                out.extend(struct.pack(">H", delta))
        elif kind == "same_locals":
            if delta <= 63:
                out.append(64 + delta)
            else:
                out.append(247)
                out.extend(struct.pack(">H", delta))
            out.extend(body)
        elif kind == "chop":
            out.append(251 - body[0])
            out.extend(struct.pack(">H", delta))
        elif kind == "same_extended":
            out.append(251)
            out.extend(struct.pack(">H", delta))
        elif kind == "append":
            out.append(251 + (len(body) and frame_local_count(body)))
            out.extend(struct.pack(">H", delta))
            out.extend(body)
        elif kind == "full":
            out.append(255)
            out.extend(struct.pack(">H", delta))
            out.extend(body)
        previous = absolute
    return bytes(out)


def skip_verification_type(payload, pos):
    tag = payload[pos]
    pos += 1
    if tag == 7:
        return pos + 2
    if tag == 8:
        return pos + 2
    if tag > 8:
        raise ValueError(f"invalid verification_type_info tag {tag}")
    return pos


def frame_local_count(body):
    pos = 0
    count = 0
    while pos < len(body):
        pos = skip_verification_type(body, pos)
        count += 1
    return count


utf8, cp_end = cp_utf8()
pos = cp_end + 6
interfaces = u2(pos)
pos += 2 + interfaces * 2
fields = u2(pos)
pos += 2
for _ in range(fields):
    attrs = u2(pos + 6)
    pos += 8
    for _ in range(attrs):
        pos += 6 + u4(pos + 2)
methods = u2(pos)
pos += 2
found = False
for _ in range(methods):
    name = utf8[u2(pos + 2)]
    attr_count = u2(pos + 6)
    pos += 8
    for _ in range(attr_count):
        attr_name = utf8[u2(pos)]
        attr_length = u4(pos + 2)
        body = pos + 6
        attr_end = body + attr_length
        if name == "run" and attr_name == "Code":
            code_length = u4(body + 4)
            code_start = body + 8
            code_end = code_start + code_length
            if data[code_start + 6] != 0xB7 or data[code_start + 9] != 0x4B:
                raise ValueError("expected invokespecial@6 followed by astore_0@9")
            # CP #53 is the existing invokestatic maybeFailBody:()V method reference.
            invocation = bytes((0xB8, 0x00, 0x35))
            data[code_start + 9:code_start + 9] = invocation
            put_u4(body + 4, code_length + 3)

            count_at = code_start + code_length + 3
            row_count = u2(count_at)
            table_at = count_at + 2
            for row in range(row_count):
                row_at = table_at + row * 8
                start, end, target, catch = struct.unpack_from(">HHHH", data, row_at)
                struct.pack_into(">HHHH", data, row_at,
                                 shifted_offset(start), shifted_offset(end),
                                 shifted_offset(target), catch)

            nested_count_at = table_at + row_count * 8
            nested_count = u2(nested_count_at)
            cursor = nested_count_at + 2
            code_size_delta = 3
            stack_maps = 0
            for _ in range(nested_count):
                name_index = u2(cursor)
                size = u4(cursor + 2)
                nested_body = cursor + 6
                nested_name = utf8[name_index]
                if nested_name == "StackMapTable":
                    stack_maps += 1
                    old = bytes(data[nested_body:nested_body + size])
                    new = rewrite_stack_map(old)
                    data[nested_body:nested_body + size] = new
                    size_delta = len(new) - size
                    put_u4(cursor + 2, len(new))
                    code_size_delta += size_delta
                    size = len(new)
                cursor = nested_body + size

            if stack_maps != 1 or nested_count != 1:
                raise ValueError("frozen run Code must have exactly one nested StackMapTable")
            put_u4(pos + 2, attr_length + code_size_delta)
            found = True
            break
        pos = attr_end
    if found:
        break
if not found:
    raise ValueError("run() Code attribute not found")

TARGET.write_bytes(data)
sha = hashlib.sha256(data).hexdigest()
(OUT / "class.sha256").write_text(f"{sha}  {TARGET.name}\n")
javap = subprocess.run(
    ["javap", "-v", "-c", "-p", str(TARGET)],
    check=True,
    capture_output=True,
    text=True,
    timeout=10,
)
start = javap.stdout.index("  private static int run()")
end = javap.stdout.index("    Exceptions:", start)
(OUT / "run-javap.txt").write_text(javap.stdout[start:end])

runtime = {}
for label, main_class in (("original", SOURCE), ("effectful", TARGET)):
    with tempfile.TemporaryDirectory(prefix=f"jarde-twr-{label}-") as tmp:
        class_dir = Path(tmp)
        for dependency in sorted((BASE / "release8").glob("MultiResourceTwr$*.class")):
            shutil.copy2(dependency, class_dir / dependency.name)
        shutil.copy2(main_class, class_dir / "MultiResourceTwr.class")
        lines = []
        for mode in ("normal", "body", "inner-close", "outer-close", "suppressed"):
            result = subprocess.run(
                ["java", "-Xverify:all", "-cp", str(class_dir), "MultiResourceTwr", mode],
                check=True,
                capture_output=True,
                text=True,
                timeout=10,
            )
            lines.append(f"{mode} {result.stdout.strip()}")
        runtime[label] = "\n".join(lines) + "\n"
(OUT / "original-runtime.txt").write_text(runtime["original"])
(OUT / "runtime.txt").write_text(runtime["effectful"])
diff = difflib.unified_diff(
    runtime["original"].splitlines(keepends=True),
    runtime["effectful"].splitlines(keepends=True),
    fromfile="original/runtime.txt",
    tofile="effectful/runtime.txt",
)
(OUT / "original-vs-effectful.diff").write_text("".join(diff))

# Independent identity mismatch: the proved construction at BCI 1 is consumed by
# Probe.close(), while the resource Store at BCI 13 consumes null. Both are
# verifier-valid references to Probe, but the constructor result is not the Store value.
data = bytearray(SOURCE.read_bytes())
utf8, cp_end = cp_utf8()
pos = cp_end + 6
interfaces = u2(pos)
pos += 2 + interfaces * 2
fields = u2(pos)
pos += 2
for _ in range(fields):
    attrs = u2(pos + 6)
    pos += 8
    for _ in range(attrs):
        pos += 6 + u4(pos + 2)
methods = u2(pos)
pos += 2
identity_out = BASE / "constructor-identity-negative"
identity_out.mkdir(exist_ok=True)
identity_target = identity_out / TARGET.name
identity_found = False
for _ in range(methods):
    name = utf8[u2(pos + 2)]
    attr_count = u2(pos + 6)
    pos += 8
    for _ in range(attr_count):
        attr_name = utf8[u2(pos)]
        attr_length = u4(pos + 2)
        body = pos + 6
        if name == "run" and attr_name == "Code":
            code_length = u4(body + 4)
            code_start = body + 8
            if data[code_start + 9] != 0x4B:
                raise ValueError("expected outer resource Store at BCI 9")
            put_u2(body, 4)  # the retained null plus the construction needs four stack slots
            data[code_start:code_start] = bytes((0x01,))
            store = code_start + 10
            data[store:store + 1] = bytes((0xB6, 0x00, 0x3B, 0x4B))
            put_u4(body + 4, code_length + 4)
            table_at = code_start + code_length + 4
            row_count = u2(table_at)
            rows_at = table_at + 2
            for row in range(row_count):
                row_at = rows_at + row * 8
                start, end, target, catch = struct.unpack_from(">HHHH", data, row_at)
                struct.pack_into(">HHHH", data, row_at,
                                 shifted_offset(start, 10, 4),
                                 shifted_offset(end, 10, 4),
                                 shifted_offset(target, 10, 4), catch)
            nested_count_at = rows_at + row_count * 8
            nested_count = u2(nested_count_at)
            cursor = nested_count_at + 2
            stack_maps = 0
            for _ in range(nested_count):
                name_index = u2(cursor)
                size = u4(cursor + 2)
                nested_body = cursor + 6
                if utf8[name_index] == "StackMapTable":
                    stack_maps += 1
                    old = bytes(data[nested_body:nested_body + size])
                    new = rewrite_stack_map(old, 10, 4)
                    data[nested_body:nested_body + size] = new
                    put_u4(cursor + 2, len(new))
                    attr_length += len(new) - size
                    size = len(new)
                cursor = nested_body + size
            if stack_maps != 1 or nested_count != 1:
                raise ValueError("frozen run Code must have exactly one nested StackMapTable")
            put_u4(pos + 2, attr_length + 4)
            identity_found = True
            break
        pos = body + attr_length
    if identity_found:
        break
if not identity_found:
    raise ValueError("run() Code attribute not found for identity negative")
identity_target.write_bytes(data)
identity_sha = hashlib.sha256(data).hexdigest()
(identity_out / "class.sha256").write_text(f"{identity_sha}  {identity_target.name}\n")
javap = subprocess.run(
    ["javap", "-v", "-c", "-p", str(identity_target)],
    check=True,
    capture_output=True,
    text=True,
    timeout=10,
)
start = javap.stdout.index("  private static int run()")
end = javap.stdout.index("    Exceptions:", start)
(identity_out / "run-javap.txt").write_text(javap.stdout[start:end])
identity_lines = []
for mode in ("normal", "body", "inner-close", "outer-close", "suppressed"):
    with tempfile.TemporaryDirectory(prefix="jarde-twr-identity-") as tmp:
        class_dir = Path(tmp)
        for dependency in sorted((BASE / "release8").glob("MultiResourceTwr$*.class")):
            shutil.copy2(dependency, class_dir / dependency.name)
        shutil.copy2(identity_target, class_dir / identity_target.name)
        result = subprocess.run(
            ["java", "-Xverify:all", "-cp", str(class_dir), "MultiResourceTwr", mode],
            check=True,
            capture_output=True,
            text=True,
            timeout=10,
        )
        identity_lines.append(f"{mode} {result.stdout.strip()}")
(identity_out / "runtime.txt").write_text("\n".join(identity_lines) + "\n")
identity_diff = difflib.unified_diff(
    runtime["original"].splitlines(keepends=True),
    (identity_out / "runtime.txt").read_text().splitlines(keepends=True),
    fromfile="original/runtime.txt",
    tofile="constructor-identity-negative/runtime.txt",
)
(identity_out / "original-vs-identity.diff").write_text("".join(identity_diff))
print(f"sha256 {sha}  {TARGET}")
print(f"sha256 {identity_sha}  {identity_target}")
