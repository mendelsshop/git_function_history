import requests
import json
import sys
from github import Github
import os
import xml.etree.ElementTree as ET

count = 0
if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <github token>")
    sys.exit(1)
crates = ["git_function_history", "cargo-function-history", "git-function-history-gui", "function_history_backend_thread"]
for crate in crates:
    jsons = requests.get(f'https://img.shields.io/crates/d/{crate}?label=crates.io%20downloads')
    root = ET.fromstring(jsons.content)
    for i in root:
        if type(i.text) is str:
            if i.text.startswith('crates.io'):
                print(f'{crate}: {i.text.split(" ")[-1]}')
                print(i.text.split(' ')[0], i.text.split(' ')[2])
                count += int(i.text.split(' ')[-1])


base64_json = {"schemaVersion":1,"label":"Crates.io Total Downloads","message":f"{count}","color":"black"}
base64_json = json.dumps(base64_json)
print(count)
g = Github(sys.argv[1])
# get last sha
git = g.get_repo("mendelsshop/git_function_history")
commit = git.get_contents("downloads.json", ref="stats")
old = commit.decoded_content
print(old)
print(base64_json.encode())
if old == base64_json.encode():
    print("same")
else: 
    # update the file
    git.update_file("downloads.json", "update downloads.json", base64_json, commit.sha, branch="stats")
    print("different")



# # using an access token
# g = Github(sys.argv[1])

# # get last sha
# git = g.get_repo("mendelsshop/git_function_history")
# commit = git.get_contents("downloads.json", ref="stats")
# old = commit.decoded_content
# print(old)
# print(base64_json.encode())
# if old == base64_json.encode():
#     print("same")
# else: 
#     # update the file
#     git.update_file("downloads.json", "update downloads.json", base64_json, commit.sha, branch="stats")
#     print("different")
