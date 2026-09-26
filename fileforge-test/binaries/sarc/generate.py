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


def write(name: str, files: dict[str, bytes], endianness: oead.Endianness) -> None:
    writer = oead.SarcWriter(endianness)
    for path, data in files.items():
        writer.files[path] = data
    _, data = writer.write()
    (HERE / name).write_bytes(data)


write("three-files-le.sarc", FILES, oead.Endianness.Little)
write("three-files-be.sarc", FILES, oead.Endianness.Big)
write("empty-le.sarc", {}, oead.Endianness.Little)
