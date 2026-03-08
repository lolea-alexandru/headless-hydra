# ========================= IMPORTS ========================= #
import random
import json
random.seed(42)

4294967295
# ========================= CONFIG ========================= #
sites = random.sample(range(22, 65500, 15), 1000)
ranges = [(i, i + random.randint(1, 14)) for i in sites]

interval_list = []

for interval in ranges:
    interval_list.append({"lower": interval[0], "upper": interval[1]})

# 
with open("../fully_HE/src/intervals.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list, write_file)
