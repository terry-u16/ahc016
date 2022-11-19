import numpy as np

TIMESTAMP = "20221119_023835"

for bits in range(4, 7):
    for eps in range(0, 41):
        m = [0, 0, 0, 0, 11, 34, 156][bits]
        CSV_PATH = f"./data/sampled/{TIMESTAMP}/{bits}_{eps:02}.csv"

        matrix = np.zeros((m, m), dtype=np.int32)

        with open(CSV_PATH, "r") as f:
            for line in f:
                truth, answered = map(int, line.split(","))
                matrix[truth, answered] += 1

        for col in range(0, m):
            edges = []
            for row in range(0, m):
                if matrix[row, col] > 0:
                    edges.append((row, matrix[row, col]))

            print(f"{len(edges):02x}", end="")
            for row, count in edges:
                print(f"{row:02x}{count:03x}", end="")
