"""Check the ARM64 daemon's load segments on 16 KiB Android kernels."""

import struct
import sys
from pathlib import Path


def check(path: Path) -> None:
    data = path.read_bytes()
    if data[:6] != b"\x7fELF\x02\x01":
        raise ValueError("Expected a little-endian ELF64 executable")
    header = struct.unpack_from("<16sHHIQQQIHHHHHH", data)
    if header[2] != 183:
        raise ValueError("Expected AArch64")
    offset, entry_size, count = header[5], header[9], header[10]
    if entry_size < 56:
        raise ValueError("Invalid program header size")
    loads = 0
    for index in range(count):
        segment = struct.unpack_from("<IIQQQQQQ", data, offset + index * entry_size)
        if segment[0] != 1:  # PT_LOAD
            continue
        loads += 1
        file_offset, address, alignment = segment[2], segment[3], segment[7]
        if alignment < 16384 or alignment & (alignment - 1):
            raise ValueError(f"LOAD segment {index} alignment is {alignment:#x}")
        if file_offset % 16384 != address % 16384:
            raise ValueError(f"LOAD segment {index} has incompatible offset/address")
    if not loads:
        raise ValueError("No LOAD segments found")
    print(f"{path}: {loads} LOAD segments support 16 KiB alignment")


if __name__ == "__main__":
    check(Path(sys.argv[1]))
