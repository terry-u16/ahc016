import json

MIN_M = 10
MAX_M = 100
MAX_EPS = 40

TIMESTAMP = "20221120_003638"

with open(f"./data/accuracy/{TIMESTAMP}/statistics.json", "r") as f:
    data = json.load(f)

max_scores = [[0] * (MAX_EPS + 1) for _ in range(MIN_M, MAX_M + 1)]
storategies = [[None] * (MAX_EPS + 1) for _ in range(MIN_M, MAX_M + 1)]

for stat in data:
    m = stat["m"] - MIN_M
    eps = int((stat["error_ratio"] * 100) + 0.01)
    score = stat["expected_score"]
    bits = stat["bits"]
    redundancy = stat["redundancy"]
    score_coef = stat["score_coef"]

    if max_scores[m][eps] < score:
        max_scores[m][eps] = score
        storategies[m][eps] = (bits, redundancy, score_coef)

for m in reversed(range(MIN_M, MAX_M)):
    for eps in range(MAX_EPS + 1):
        if storategies[m - MIN_M][eps] == None:
            storategies[m - MIN_M][eps] = storategies[m - MIN_M + 1][eps]

for m in range(MIN_M, MAX_M + 1):
    for eps in range(MAX_EPS + 1):
        bits, redundancy, score_coef = storategies[m - MIN_M][eps]
        score_coef = int(score_coef * 10 + 0.1)
        print(f"{bits:01}{redundancy:02}{score_coef:02}", end="")
