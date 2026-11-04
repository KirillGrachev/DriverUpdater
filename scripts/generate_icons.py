#!/usr/bin/env python3
"""Generate the Driver Updater icons (PNGs + ICO) without PIL.

Renders a rounded square with a blue diagonal gradient and a white gear
(ring + eight teeth) using pure per-pixel math.
"""
import os
import struct
import zlib

OUT_DIR = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")


def lerp(a, b, t):
    return a + (b - a) * t


def gear_mask(px, py, size):
    """1.0 inside the gear (ring + teeth) at continuous coords."""
    import math

    c = size / 2.0
    dx, dy = px - c, py - c
    r = (dx * dx + dy * dy) ** 0.5
    ring_outer = size * 0.30
    ring_inner = size * 0.155
    tooth_outer = size * 0.385
    if ring_inner <= r <= ring_outer:
        return 1.0
    if ring_outer < r <= tooth_outer:
        angle = math.atan2(dy, dx) % (math.pi / 4)
        tooth_half = math.pi / 14
        return 1.0 if abs(angle - math.pi / 8) <= tooth_half else 0.0
    return 0.0


def rounded_square(px, py, size):
    """1.0 inside the rounded square at continuous coords (px, py)."""
    radius = size * 0.22
    c = size / 2.0
    dx, dy = abs(px - c) - (c - radius), abs(py - c) - (c - radius)
    if dx <= 0 and dy <= 0:
        return 1.0
    if dx <= 0:
        return 1.0 if dy <= radius else 0.0
    if dy <= 0:
        return 1.0 if dx <= radius else 0.0
    return 1.0 if (dx * dx + dy * dy) <= radius * radius else 0.0


def render(size):
    pixels = bytearray()
    top = (37, 99, 235)  # blue-600
    bottom = (29, 78, 216)  # blue-700
    accent = (96, 165, 250)  # blue-400 highlight
    samples = [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
    for y in range(size):
        for x in range(size):
            t = (x + y) / (2.0 * size)
            base = [lerp(top[i], bottom[i], t) for i in range(3)]
            base = [lerp(base[i], accent[i], max(0.0, 0.35 - t)) for i in range(3)]
            mask_sum = 0.0
            gear_sum = 0.0
            for sx, sy in samples:
                mask_sum += rounded_square(x + sx, y + sy, size)
                gear_sum += gear_mask(x + sx, y + sy, size)
            mask = mask_sum / len(samples)
            gear = gear_sum / len(samples)
            rgb = [lerp(base[i], 255.0, gear) for i in range(3)]
            pixels.extend([int(c) for c in rgb] + [int(mask * 255)])
    return bytes(pixels)


def png_chunk(tag, data):
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def write_png(path, size):
    raw = render(size)
    scanlines = bytearray()
    stride = size * 4
    for y in range(size):
        scanlines.append(0)
        scanlines.extend(raw[y * stride : (y + 1) * stride])
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n"
    png += png_chunk(b"IHDR", ihdr)
    png += png_chunk(b"IDAT", zlib.compress(bytes(scanlines), 9))
    png += png_chunk(b"IEND", b"")
    with open(path, "wb") as fh:
        fh.write(png)


def write_ico(path, size):
    raw = render(size)
    scanlines = bytearray()
    stride = size * 4
    for y in range(size):
        scanlines.append(0)
        scanlines.extend(raw[y * stride : (y + 1) * stride])
    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n"
    png += png_chunk(b"IHDR", ihdr)
    png += png_chunk(b"IDAT", zlib.compress(bytes(scanlines), 9))
    png += png_chunk(b"IEND", b"")
    header = struct.pack("<HHH", 0, 1, 1)
    entry = struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(png), 22)
    with open(path, "wb") as fh:
        fh.write(header + entry + png)


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    write_png(os.path.join(OUT_DIR, "32x32.png"), 32)
    write_png(os.path.join(OUT_DIR, "128x128.png"), 128)
    write_png(os.path.join(OUT_DIR, "128x128@2x.png"), 256)
    write_ico(os.path.join(OUT_DIR, "icon.ico"), 32)
    print("icons written to", OUT_DIR)


if __name__ == "__main__":
    main()
