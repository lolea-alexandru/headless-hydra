# ========================= IMPORTS ========================= #
import random
import json
random.seed(42)

4294967295
# ========================= CONFIG ========================= #
sites = random.sample(range(22, 100000000, 25), 50000)
ranges = [(i, i + random.randint(1, 20)) for i in sites]

print(ranges)

interval_list = []

for interval in ranges:
    interval_list.append({"start": interval[0], "end": interval[1]})

with open("intervals.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list, write_file)
