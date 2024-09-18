import numpy as np
import cv2
import requests
import os
from collections import defaultdict
from scipy import stats
from collections import Counter
import pickle
import json
from pdf2image import convert_from_path

start_page = 7
end_page = 33
radius = 12

colors = {
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

shapes = {
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
    "red": np.array(
        [
            [True, True],
            [True, True],
            [True, False],
        ]
    ),
}

color_count = {
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


def color_distance(c1, c2):
    return np.sqrt(sum((a - b) ** 2 for a, b in zip(c1, c2)))


def find_closest_color(most_common_color):
    if isinstance(most_common_color, np.ndarray):
        most_common_color = tuple(most_common_color.astype(int))
    elif isinstance(most_common_color, list):
        most_common_color = tuple(most_common_color)

    closest_color_name = None
    min_distance = float("inf")

    for color_name, color_rgb in colors.items():
        distance = color_distance(most_common_color, color_rgb)
        if distance < min_distance:
            min_distance = distance
            closest_color_name = color_name

    return closest_color_name


def create_color_grid(colors_list, grid_width, grid_height):
    colors_list = sorted(colors_list, key=lambda x: x[1])
    grid = [
        sorted(
            colors_list[h * grid_width : h * grid_width + grid_width],
            key=lambda x: x[0],
        )
        for h in range(0, grid_height)
    ]
    return grid


def get_most_common_color(image, center, radius):
    height, width, _ = image.shape
    mask = np.zeros((height, width), dtype=np.uint8)
    cv2.circle(mask, center, radius, 255, thickness=-1)
    masked_image = cv2.bitwise_and(image, image, mask=mask)
    pixels = masked_image[mask > 0]
    if len(pixels) == 0:
        return None
    pixels_list = [tuple(pixel) for pixel in pixels]
    color_counts = Counter(pixels_list)
    mcc = color_counts.most_common(1)[0][0]
    for k, v in colors.items():
        if (mcc[2], mcc[1], mcc[0]) == v:
            return k
    assert False


def rotate_90(matrix):
    return np.rot90(matrix, -1)


def flip_horizontal(matrix):
    return np.fliplr(matrix)


def flip_vertical(matrix):
    return np.flipud(matrix)


def get_all_transformations(shape):
    transformations = []
    for _ in range(4):
        transformations.append(shape)
        transformations.append(flip_horizontal(shape))
        transformations.append(flip_vertical(shape))
        transformations.append(flip_horizontal(flip_vertical(shape)))
        shape = rotate_90(shape)
    return transformations


def print_grid(grid):
    for row in grid:
        print(" ".join(row))
    print()

    filled_grid = np.full_like(grid, "DARK_GRAY")


def check_shape_in_grid(transformation, grid, row, col, name):
    shape = np.array(transformation["shape"])
    shape_height, shape_width = shape.shape
    if row + shape_height > grid.shape[0] or col + shape_width > grid.shape[1]:
        return False
    subgrid = grid[row : row + shape_height, col : col + shape_width]
    subgrid = np.where(subgrid == name.upper(), True, False)

    return np.array_equal(subgrid, shape)


def find_shape_location(shape_name, grid, transformation):
    for row in range(grid.shape[0]):
        for col in range(grid.shape[1]):
            for idx, transformation in enumerate(transformations):
                if check_shape_in_grid(transformation, grid, row, col, shape_name):
                    return (shape_name, row, col, idx)
    return (shape_name, None, None, None)


def get_all_transformations_json(shape):
    transformations = []
    current = shape
    for rotation in range(4):
        transformations.append(
            {
                "rotation": rotation * 90,
                "flip_horizontal": False,
                "flip_vertical": False,
                "shape": current.tolist(),
            }
        )
        transformations.append(
            {
                "rotation": rotation * 90,
                "flip_horizontal": True,
                "flip_vertical": False,
                "shape": flip_horizontal(current).tolist(),
            }
        )
        transformations.append(
            {
                "rotation": rotation * 90,
                "flip_horizontal": False,
                "flip_vertical": True,
                "shape": flip_vertical(current).tolist(),
            }
        )
        transformations.append(
            {
                "rotation": rotation * 90,
                "flip_horizontal": True,
                "flip_vertical": True,
                "shape": flip_horizontal(flip_vertical(current)).tolist(),
            }
        )
        current = rotate_90(current)
    return transformations


pages = convert_from_path("KanoodleGuide.pdf")
for page_num, page in enumerate(pages):
    if page_num >= start_page and page_num <= end_page:
        page.save(f"pages/page{page_num}.png", "png")

page_to_puzzles = defaultdict(list)

for page in os.listdir("pages"):
    page_num = int("".join([x for x in page if x.isdigit()])) - start_page
    img = cv2.imread(f"pages/{page}")
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    ret, thresh = cv2.threshold(gray, 127, 255, 0)

    contours, h = cv2.findContours(thresh, 1, cv2.CHAIN_APPROX_SIMPLE)

    i = 0
    for cnt in contours:
        approx = cv2.approxPolyDP(cnt, 0.01 * cv2.arcLength(cnt, True), True)
        if len(approx) == 4:
            if i > 0:
                i += 1
                x, y, w, h = cv2.boundingRect(cnt)

                if w * h > 35000:
                    ROI = img[y : y + h, x : x + w]
                    page_to_puzzles[page_num].append((x, y, ROI))
            else:
                i += 1

for k, v in page_to_puzzles.items():
    v = sorted(v, key=lambda e: (e[1], e[0]))
    for puzzle_num in range(0, len(v)):
        cv2.imwrite(f"puzzles/puzzle{k * 6 + puzzle_num}.png", v[puzzle_num][2])


puzzle_raw = {}

last = None

puzzle_to_config = defaultdict(list)

for puzzle in os.listdir("puzzles"):
    last = puzzle
    puzzle_num = int("".join([x for x in puzzle if x.isdigit()]))

    img = cv2.imread(f"puzzles/{puzzle}")
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    circles = cv2.HoughCircles(
        gray, cv2.HOUGH_GRADIENT, 1, 10, param1=50, param2=20, minRadius=9, maxRadius=14
    )

    sorted_circles = []

    if circles is not None:
        circles = np.round(circles[0, :]).astype("int")

    if len(circles) != 55:
        print(f"{puzzle} was not detected correctly")
        continue

    for x, y, r in circles:
        sorted_circles.append((x, y))
    grid = create_color_grid(list(sorted_circles), 11, 5)

    color_grid = [[None for _ in range(11)] for _ in range(5)]

    for r in range(len(grid)):
        for i, center in enumerate(grid[r]):
            most_common_color = get_most_common_color(img, center, radius)
            color_grid[r][i] = most_common_color

    flatten = sum(color_grid, [])
    counter = Counter(flatten)

    for color, count in color_count.items():
        assert counter[color] == 0 or counter[color] == count

    puzzle_raw[puzzle] = color_grid

    grid = np.array(color_grid)

    for shape_name, shape in shapes.items():
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


all_shapes_transformations = {}

for shape_name, shape in shapes.items():
    transformations = get_all_transformations_json(shape)
    all_shapes_transformations[shape_name] = transformations

with open("json/shapes_transformations.json", "w") as f:
    json.dump(all_shapes_transformations, f, indent=4)
print("Transformations have been saved to 'json/shapes_transformations.json'.")

with open("json/puzzle_config.json", "w") as f:
    json.dump(puzzle_to_config, f, indent=4)
print("Puzzle configs have been saved to 'json/puzzle_config.json'.")
