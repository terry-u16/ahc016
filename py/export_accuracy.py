import numpy as np

TIMESTAMP = "20221119_023835"
M_LIST = [0, 0, 0, 0, 11, 34, 156]

values = []

for bits in range(4, 7):
    m = M_LIST[bits]

    for eps in range(0, 41):
        csv_path = f"./data/sampled/{TIMESTAMP}/{bits}_{eps:02}.csv"
        matrix = np.zeros((m, m), dtype=np.int32)

        with open(csv_path, "r") as f:
            for line in f:
                truth, answered = map(int, line.split(","))
                matrix[truth, answered] += 1

        diag = np.diag(matrix)

        for d in diag:
            values.append(f"{d:03x}")

print("".join(values))
