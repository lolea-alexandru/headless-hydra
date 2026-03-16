# ========================= IMPORTS ========================= #
import random
import json
random.seed(42)
    
# ========================= CONFIG ========================= #
sites_first_interval = random.sample(range(22, 65500, 15), 40)
ranges_first_interval = [(i, i + random.randint(1, 14)) for i in sites_first_interval]

sites_second_interval = random.sample(range(22, 65500, 10), 40)
ranges_second_interval = [(i, i + random.randint(1, 8)) for i in sites_second_interval]

first_interval_list = []
second_interval_list = []

for interval in ranges_first_interval:
    first_interval_list.append({"lower": interval[0], "upper": interval[1]})

for interval in ranges_second_interval:
    second_interval_list.append({"lower": interval[0], "upper": interval[1]})
# 

# TODO: Maake the script more flexible in terms of choosing paths
with open("../basic-diffie-hellman/src/intervals_1.json", mode="w", encoding="utf-8") as write_file:
    json.dump(first_interval_list, write_file)
with open("../basic-diffie-hellman/src/intervals_2.json", mode="w", encoding="utf-8") as write_file:
    json.dump(second_interval_list, write_file)
