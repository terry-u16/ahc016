import base64
import os
import struct


def pack(value: float) -> bytes:
    return struct.pack("<d", value)


def pack_float_sequence(values: list[float]) -> bytes:
    stream = bytearray([])

    for v in values:
        stream.extend(pack(v))

    return base64.b64encode(stream)
