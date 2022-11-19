import json

MIN_M = 10
MAX_M = 100
MAX_EPS = 40

TIMESTAMP = "20221119_192902"

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

    if max_scores[m][eps] < score:
        max_scores[m][eps] = score
        storategies[m][eps] = (bits, redundancy)

for m in reversed(range(MIN_M, MAX_M)):
    for eps in range(MAX_EPS + 1):
        if storategies[m - MIN_M][eps] == None:
            storategies[m - MIN_M][eps] = storategies[m - MIN_M + 1][eps]

for m in range(MIN_M, MAX_M + 1):
    for eps in range(MAX_EPS + 1):
        bits, redundancy = storategies[m - MIN_M][eps]
        print(f"{bits:01}{redundancy:02}", end="")