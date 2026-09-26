"""Generates the SARC test archives with oead, an independent implementation.

    python -m pip install oead
    python fileforge-test/binaries/sarc/generate.py
"""

from pathlib import Path

import oead

HERE = Path(__file__).parent

FILES = {
    "Model/Actor.bfres": bytes(range(40)),
    "Param/Actor.byml": b"BY" + bytes(18),
    "Readme.txt": b"hello, sarc\n",
}


def write(name: str, files: dict[str, bytes], endianness: oead.Endianness, number_collisions: bool = False) -> None:
    writer = oead.SarcWriter(endianness)
    for path, data in files.items():
        writer.files[path] = data
    _, data = writer.write()
    if number_collisions:
        data = numbered(data)
    (HERE / name).write_bytes(data)


def numbered(data: bytes) -> bytes:
    """Numbers entries that share a hash 1, 2, 3... in the top byte of their name attributes, as
    sead expects. oead writes 1 for all of them (and its reader ignores collisions)."""
    data = bytearray(data)
    order = "little" if data[6:8] == bytes([0xFF, 0xFE]) else "big"
    count = int.from_bytes(data[0x1A:0x1C], order)
    previous, sequence = None, 0
    for index in range(count):
        entry = 0x20 + 16 * index
        hash = data[entry : entry + 4]
        sequence = sequence + 1 if hash == previous else 1
        previous = hash
        attributes = int.from_bytes(data[entry + 4 : entry + 8], order)
        data[entry + 4 : entry + 8] = ((sequence << 24) | (attributes & 0xFFFFFF)).to_bytes(4, order)
    return bytes(data)


write("three-files-le.sarc", FILES, oead.Endianness.Little)
write("three-files-be.sarc", FILES, oead.Endianness.Big)
write("empty-le.sarc", {}, oead.Endianness.Little)

# Two pairs of names that share a sead hash (found by a random search), among other files: once
# as oead writes them, and once numbered the way sead expects.
COLLISIONS = {
    "c/aycpvjtnn": b"first of the first pair",
    "c/kevbqeqbp": b"second of the first pair",
    "c/gwbuwf": b"first of the second pair",
    "c/lskfark": b"second of the second pair",
    **FILES,
}
write("collisions-oead-le.sarc", COLLISIONS, oead.Endianness.Little)
write("collisions-le.sarc", COLLISIONS, oead.Endianness.Little, number_collisions=True)

# A name with bytes over 0x7F, which sead hashes as signed.
write("unicode-le.sarc", {"Текст.txt": b"unicode", **FILES}, oead.Endianness.Little)
