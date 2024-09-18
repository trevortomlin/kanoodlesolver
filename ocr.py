import numpy as np
import cv2
import os
import json
from collections import defaultdict, Counter
from pdf2image import convert_from_path
from typing import List, Tuple

START_PAGE = 7
END_PAGE = 33
RADIUS = 12
PAGES_DIR = "pages"
PUZZLES_DIR = "puzzles"
JSON_DIR = "json"

COLORS = {
    "GREEN": (0, 171, 79),
    "CYAN": (140, 225, 249),
    "YELLOW": (255, 235, 61),
    "PURPLE": (152, 34, 125),
    "ORANGE": (255, 125, 36),
    "RED": (255, 46, 23),
    "BLUE": (0, 144, 211),
    "GREY": (196, 196, 199),
    "MAGENTA": (252, 92, 172),
    "PINK": (255, 213, 206),
    "YELLOW_GREEN": (190, 214, 67),
    "DARK_GRAY": (101, 102, 107),
    "OFF_WHITE": (247, 243, 227),
    "DARKER_GRAY": (44, 46, 53),
    "BLACK": (0, 0, 0),
    "WHITE": (255, 255, 255),
    "LIGHT_GRAY": (176, 177, 179),
}

SHAPES = {
    "orange": np.array([[True, True, True], [True, False, False]]),
    "blue": np.array([[True, True, True, True], [True, False, False, False]]),
    "pink": np.array([[True, True, True, True], [False, True, False, False]]),
    "magenta": np.array(
        [[True, True, False], [False, True, True], [False, False, True]]
    ),
    "cyan": np.array([[True, True, True], [True, False, False], [True, False, False]]),
    "light_gray": np.array(
        [[False, True, False], [True, True, True], [False, True, False]]
    ),
    "yellow_green": np.array([[True, True], [True, True]]),
    "yellow": np.array(
        [[True, True, False], [True, False, False], [True, True, False]]
    ),
    "off_white": np.array([[True, False], [True, True]]),
    "green": np.array([[True, True, False, False], [False, True, True, True]]),
    "purple": np.array([[True, True, True, True]]),
    "red": np.array([[True, True], [True, True], [True, False]]),
}

COLOR_COUNT = {
    "ORANGE": 4,
    "BLUE": 5,
    "PINK": 5,
    "MAGENTA": 5,
    "CYAN": 5,
    "LIGHT_GRAY": 5,
    "YELLOW_GREEN": 4,
    "YELLOW": 5,
    "OFF_WHITE": 3,
    "GREEN": 5,
    "PURPLE": 4,
    "RED": 5,
}


def color_distance(c1: Tuple, c2: Tuple) -> float:
    return np.sqrt(sum((a - b) ** 2 for a, b in zip(c1, c2)))


def find_closest_color(most_common_color):
    if isinstance(most_common_color, (np.ndarray, list)):
        most_common_color = tuple(map(int, most_common_color))
    return min(
        COLORS, key=lambda color: color_distance(most_common_color, COLORS[color])
    )


def create_color_grid(colors_list: List, grid_width: int, grid_height: int) -> List:
    colors_list = sorted(colors_list, key=lambda x: x[1])
    return [
        sorted(
            colors_list[h * grid_width : h * grid_width + grid_width],
            key=lambda x: x[0],
        )
        for h in range(grid_height)
    ]


def get_most_common_color(image, center, radius):
    height, width, _ = image.shape
    mask = np.zeros((height, width), dtype=np.uint8)
    cv2.circle(mask, center, radius, 255, thickness=-1)
    masked_image = cv2.bitwise_and(image, image, mask=mask)
    pixels = masked_image[mask > 0]
    if not pixels.size:
        return None
    pixels_list = [tuple(pixel) for pixel in pixels]
    most_common_color = Counter(pixels_list).most_common(1)[0][0]
    for name, rgb in COLORS.items():
        if (most_common_color[2], most_common_color[1], most_common_color[0]) == rgb:
            return name
    raise ValueError("Color not found in predefined list")


def rotate_90(matrix: np.ndarray) -> np.ndarray:
    return np.rot90(matrix, -1)


def flip_horizontal(matrix: np.ndarray) -> np.ndarray:
    return np.fliplr(matrix)


def flip_vertical(matrix: np.ndarray) -> np.ndarray:
    return np.flipud(matrix)


def get_all_transformations(shape: np.ndarray) -> List:
    transformations = []
    for _ in range(4):
        transformations.extend(
            [
                shape,
                flip_horizontal(shape),
                flip_vertical(shape),
                flip_horizontal(flip_vertical(shape)),
            ]
        )
        shape = rotate_90(shape)
    return transformations


def print_grid(grid: List) -> None:
    for row in grid:
        print(" ".join(row))
    print()


def check_shape_in_grid(
    transformation, grid: List, row: int, col: int, name: str
) -> bool:
    shape = np.array(transformation["shape"])
    shape_height, shape_width = shape.shape
    if row + shape_height > grid.shape[0] or col + shape_width > grid.shape[1]:
        return False
    subgrid = grid[row : row + shape_height, col : col + shape_width]
    subgrid = np.where(subgrid == name.upper(), True, False)
    return np.array_equal(subgrid, shape)


def find_shape_location(
    shape_name: str, grid: List, transformations: List
) -> (str, int, int, int):
    for row in range(grid.shape[0]):
        for col in range(grid.shape[1]):
            for idx, transformation in enumerate(transformations):
                if check_shape_in_grid(transformation, grid, row, col, shape_name):
                    return shape_name, row, col, idx
    return shape_name, None, None, None


def get_all_transformations_json(shape: np.ndarray) -> List:
    transformations = []
    current = shape
    for rotation in range(4):
        transformations.extend(
            [
                {
                    "rotation": rotation * 90,
                    "flip_horizontal": False,
                    "flip_vertical": False,
                    "shape": current.tolist(),
                },
                {
                    "rotation": rotation * 90,
                    "flip_horizontal": True,
                    "flip_vertical": False,
                    "shape": flip_horizontal(current).tolist(),
                },
                {
                    "rotation": rotation * 90,
                    "flip_horizontal": False,
                    "flip_vertical": True,
                    "shape": flip_vertical(current).tolist(),
                },
                {
                    "rotation": rotation * 90,
                    "flip_horizontal": True,
                    "flip_vertical": True,
                    "shape": flip_horizontal(flip_vertical(current)).tolist(),
                },
            ]
        )
        current = rotate_90(current)
    return transformations


def main() -> None:
    pages = convert_from_path("KanoodleGuide.pdf")
    for page_num, page in enumerate(pages):
        if START_PAGE <= page_num <= END_PAGE:
            page.save(f"{PAGES_DIR}/page{page_num}.png", "png")

    page_to_puzzles = defaultdict(list)
    for page_file in os.listdir(PAGES_DIR):
        page_num = int("".join(filter(str.isdigit, page_file))) - START_PAGE
        img = cv2.imread(f"{PAGES_DIR}/{page_file}")
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        ret, thresh = cv2.threshold(gray, 127, 255, 0)
        contours, _ = cv2.findContours(thresh, 1, cv2.CHAIN_APPROX_SIMPLE)

        for cnt in contours:
            approx = cv2.approxPolyDP(cnt, 0.01 * cv2.arcLength(cnt, True), True)
            if len(approx) == 4:
                x, y, w, h = cv2.boundingRect(cnt)
                if w * h > 35000:
                    ROI = img[y : y + h, x : x + w]
                    page_to_puzzles[page_num].append((x, y, ROI))

    for k, v in page_to_puzzles.items():
        v = sorted(v, key=lambda e: (e[1], e[0]))
        for puzzle_num, (_, _, roi) in enumerate(v):
            cv2.imwrite(f"{PUZZLES_DIR}/puzzle{k * 6 + puzzle_num}.png", roi)

    puzzle_raw = {}
    puzzle_to_config = defaultdict(list)
    for puzzle_file in os.listdir(PUZZLES_DIR):
        puzzle_num = int("".join(filter(str.isdigit, puzzle_file)))
        img = cv2.imread(f"{PUZZLES_DIR}/{puzzle_file}")
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        circles = cv2.HoughCircles(
            gray,
            cv2.HOUGH_GRADIENT,
            1,
            10,
            param1=50,
            param2=20,
            minRadius=9,
            maxRadius=14,
        )

        if circles is not None:
            circles = np.round(circles[0, :]).astype("int")
        if len(circles) != 55:
            print(f"{puzzle_file} was not detected correctly")
            continue

        sorted_circles = sorted((x, y) for x, y, _ in circles)
        grid = create_color_grid(sorted_circles, 11, 5)

        color_grid = [[None] * 11 for _ in range(5)]
        for r, row in enumerate(grid):
            for i, center in enumerate(row):
                most_common_color = get_most_common_color(img, center, RADIUS)
                color_grid[r][i] = most_common_color

        flatten = [color for row in color_grid for color in row]
        counter = Counter(flatten)
        for color, count in COLOR_COUNT.items():
            assert counter[color] in {0, count}

        puzzle_raw[puzzle_file] = color_grid
        grid = np.array(color_grid)

        for shape_name, shape in SHAPES.items():
            transformations = get_all_transformations_json(shape)
            shape_info = find_shape_location(shape_name, grid, transformations)
            name, row, col, transformation_index = shape_info
            if row is not None and col is not None and transformation_index is not None:
                puzzle_to_config[puzzle_num].append(
                    {
                        "piece": name,
                        "x": col,
                        "y": row,
                        "transformation": transformations[transformation_index],
                    }
                )

    all_shapes_transformations = {
        shape_name: get_all_transformations_json(shape)
        for shape_name, shape in SHAPES.items()
    }

    os.makedirs(JSON_DIR, exist_ok=True)
    with open(f"{JSON_DIR}/shapes_transformations.json", "w") as f:
        json.dump(all_shapes_transformations, f, indent=4)
    print("Transformations have been saved to 'json/shapes_transformations.json'.")

    with open(f"{JSON_DIR}/puzzle_config.json", "w") as f:
        json.dump(puzzle_to_config, f, indent=4)
    print("Puzzle configs have been saved to 'json/puzzle_config.json'.")


if __name__ == "__main__":
    main()
