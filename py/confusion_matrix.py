import matplotlib.pyplot as plt
import numpy as np
import seaborn as sns

TIMESTAMP = "20221119_023835"
BITS = 6
EPS = 35
M = [0, 0, 0, 0, 11, 34, 156][BITS]
CSV_PATH = f"./data/sampled/{TIMESTAMP}/{BITS}_{EPS:02}.csv"

matrix = np.zeros((M, M))

with open(CSV_PATH, "r") as f:
    for line in f:
        truth, answered = map(int, line.split(","))
        matrix[truth, answered] += 1

matrix /= matrix[0, :].sum()
diag = np.diag(matrix)

print(matrix)
print(diag)

TAKE = min(30, M)
diag_naive = np.sort(diag[:TAKE])[::-1]
print(diag_naive)
print(diag_naive.sum() / TAKE)

diag_sorted = np.sort(diag)[::-1][:TAKE]
print(diag_sorted)
print(diag_sorted.sum() / TAKE)

sns.heatmap(matrix)
plt.show()

# 対数表示
#matrix = np.log10(matrix + 1e-10)
#sns.heatmap(matrix)
#plt.show()
