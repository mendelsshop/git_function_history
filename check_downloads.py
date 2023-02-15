
import requests
import toml
import json
import sys
from github import Github
import os

if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <github token>")
    sys.exit(1)
# parse cargo.toml file and get list of members
os.system("git switch main")
toml_file = toml.load("Cargo.toml")
members = toml_file["workspace"]["members"]

count = 0
# iterate through members and use https://crates.io/api/v1/crates/{member}/downloads to get download count

# parse the download count which has to parts: meta and version_downloads
# under meta there is a section called extra_downloads which has a list of number which contain a  download count and date

# the version_downloads section contains a numbered list of a date, download count and version

for member in members:
    # get the crates name from its Cargo.toml file
    cargo_toml_file = toml.load(f"{member}/Cargo.toml")
    crate_name = cargo_toml_file["package"]["name"]
    print(f"crate name: {crate_name}")
    jsons = requests.get(f"https://crates.io/api/v1/crates/{crate_name}").json()
    print(jsons)
    # get the download count
    downloads = jsons["crate"]["downloads"]
    print(f"downloads: {downloads}")
    count += int(downloads)
os.system("git switch stats")
print(f"Total: {count}")

# upload the results to https://github.com/mendelsshop/git_function_history/stats/downloads.json
# with this format: {"schemaVersion":1,"label":"Crates.io Total Downloads","message":"0","color":"black"}
base64_json = {"schemaVersion":1,"label":"Crates.io Total Downloads","message":f"{count}","color":"black"}
base64_json = json.dumps(base64_json)

# using an access token
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
    # get the message from old
    old = json.loads(old)
    old = int(old['message'])
    if old > count:
        print("old is bigger")
        exit()
    git.update_file("downloads.json", "update downloads.json", base64_json, commit.sha, branch="stats")
    print("different")