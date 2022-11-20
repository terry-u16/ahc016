import numpy as np
import json
import matplotlib.pyplot as plt
import seaborn as sns

MIN_M = 10
MAX_M = 100
MAX_EPS = 40


def load(time_stamp: str) -> np.ndarray:
    with open(f"./data/accuracy/{time_stamp}/statistics.json", "r") as f:
        data = json.load(f)

    max_scores = np.zeros((MAX_M - MIN_M + 1, MAX_EPS + 1))

    for stat in data:
        m = stat["m"] - MIN_M
        eps = int((stat["error_ratio"] * 100) + 0.01)
        score = stat["expected_score"]

        if max_scores[m, eps] < score:
            max_scores[m, eps] = score

    for m in reversed(range(MIN_M, MAX_M)):
        for eps in range(MAX_EPS + 1):
            if max_scores[m - MIN_M, eps] == 0:
                max_scores[m - MIN_M, eps] = max_scores[m - MIN_M + 1, eps]

    return max_scores

TIMESTAMP1 = "20221120_024611"
TIMESTAMP2 = "20221120_124623"

scores1 = load(TIMESTAMP1)
scores2 = load(TIMESTAMP2)

max_scores = np.maximum(scores1, scores2)
scores1 /= max_scores
scores2 /= max_scores

count = ((MAX_M - MIN_M + 1) * (MAX_EPS + 1))
score1 = np.sum(scores1) / count
score2 = np.sum(scores2) / count

print(f"{score1} -> {score2}")

fig, axes = plt.subplots(1, 2, figsize=(20, 7))

sns.heatmap(scores1, ax=axes[0])
sns.heatmap(scores2, ax=axes[1])
plt.show()