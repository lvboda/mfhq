import re

SCREEN_ORDER = ["lt", "rt", "lb", "rb"]
SCREEN_LABELS = {
    "lt": "left-top",
    "rt": "right-top",
    "lb": "left-bottom",
    "rb": "right-bottom",
}
SCREEN_ALIASES = {
    "1": "lt",
    "lt": "lt",
    "left-top": "lt",
    "left_top": "lt",
    "zuoshang": "lt",
    "2": "rt",
    "rt": "rt",
    "right-top": "rt",
    "right_top": "rt",
    "youshang": "rt",
    "3": "lb",
    "lb": "lb",
    "left-bottom": "lb",
    "left_bottom": "lb",
    "zuoxia": "lb",
    "4": "rb",
    "rb": "rb",
    "right-bottom": "rb",
    "right_bottom": "rb",
    "youxia": "rb",
}
SCREEN_SEPARATORS_PATTERN = re.compile(r"[,，、/|;；\\-]+")


def get_split_regions(screen_size=None):
    if screen_size is None:
        import pyautogui

        screen_size = pyautogui.size()
    screen_w, screen_h = screen_size
    half_w = screen_w // 2
    half_h = screen_h // 2
    return {
        "lt": (0, 0, half_w, half_h),
        "rt": (half_w, 0, screen_w - half_w, half_h),
        "lb": (0, half_h, half_w, screen_h - half_h),
        "rb": (half_w, half_h, screen_w - half_w, screen_h - half_h),
    }


def normalize_screen_key(value):
    key = value.strip().lower()
    if key not in SCREEN_ALIASES:
        valid = ", ".join(SCREEN_ORDER)
        raise ValueError(f"invalid screen '{value}', expected one of: {valid}, 1, 2, 3, 4")
    return SCREEN_ALIASES[key]


def parse_screen_selection(value):
    if value is None:
        return list(SCREEN_ORDER)
    if isinstance(value, (list, tuple)):
        raw_parts = value
    else:
        raw_parts = [value]

    selected = []
    for raw_part in raw_parts:
        if not raw_part or raw_part.strip().lower() in {"all", "*"}:
            return list(SCREEN_ORDER)
        raw_part = raw_part.strip()
        if raw_part.lower() in SCREEN_ALIASES:
            parts = [raw_part]
        else:
            parts = SCREEN_SEPARATORS_PATTERN.split(raw_part)
        for part in parts:
            if not part.strip():
                continue
            key = normalize_screen_key(part)
            if key not in selected:
                selected.append(key)
    return selected


def describe_regions(regions, selected=None):
    selected = selected or SCREEN_ORDER
    lines = []
    for key in selected:
        x, y, w, h = regions[key]
        lines.append(f"{key}({SCREEN_LABELS[key]}): x={x}, y={y}, w={w}, h={h}")
    return "; ".join(lines)
