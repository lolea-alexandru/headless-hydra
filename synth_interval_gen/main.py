# ========================= IMPORTS ========================= #
import random
import json
random.seed(42)
    
# ========================= CONFIG ========================= #
sites_first_interval = random.sample(range(22, 65500, 15), 40)
ranges_first_interval = [(i, i + random.randint(1, 14)) for i in sites_first_interval]

interval_list_40 = []
interval_list_20 = []
interval_list_10 = []
interval_list_1 = []


for interval in ranges_first_interval:
    interval_list_40.append({"lower": interval[0], "upper": interval[1]})

# Takes the first 20 elements
interval_list_20 = interval_list_40[:20]

# Takes the first 10 elements
interval_list_10 = interval_list_40[:10]

# Takes the first 1 element
interval_list_1 = interval_list_40[:1]

# TODO: Maake the script more flexible in terms of choosing paths
with open("../experiment_intervals/intervals_40.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list_40, write_file)
with open("../experiment_intervals/intervals_20.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list_20, write_file)

# Write the 10-element list
with open("../experiment_intervals/intervals_10.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list_10, write_file)

# Write the 1-element list 
with open("../experiment_intervals/intervals_single.json", mode="w", encoding="utf-8") as write_file:
    json.dump(interval_list_1, write_file)