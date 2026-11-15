import argparse
from pathlib import Path

from PIL import Image

MIN_CHANNEL_SPREAD = 48
MAX_CHANNEL_VALUE = 235
MIN_PRIMARY_PIXELS = 100
MIN_RETAINED_FRACTION = 0.95


def primary_pixels(screenshot: Path, bounds: tuple[int, ...]) -> int:
    with Image.open(screenshot) as image:
        button = image.convert('RGB').crop(bounds)
        return sum(
            max(pixel) - min(pixel) >= MIN_CHANNEL_SPREAD
            and max(pixel) < MAX_CHANNEL_VALUE
            for pixel in button.getdata()
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('unfocused', type=Path)
    parser.add_argument('focused', type=Path)
    parser.add_argument('bounds', metavar='BOUND', nargs=4, type=int)
    args = parser.parse_args()

    baseline = primary_pixels(args.unfocused, tuple(args.bounds))
    focused = primary_pixels(args.focused, tuple(args.bounds))
    if baseline < MIN_PRIMARY_PIXELS or focused < baseline * MIN_RETAINED_FRACTION:
        raise SystemExit(f'Primary fill pixels before and after focusing: {baseline}, {focused}')


if __name__ == '__main__':
    main()
