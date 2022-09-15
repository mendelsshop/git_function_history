
import requests
import toml
import json
import sys
from github import Github

if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <github token>")
    sys.exit(1)
# parse cargo.toml file and get list of members

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
    jsons = requests.get(f"https://crates.io/api/v1/crates/{crate_name}/downloads").json()
    for i in jsons["meta"]["extra_downloads"]:
        count += i['downloads']
    for i in jsons["version_downloads"]:
        count += i['downloads']

print(f"Total: {count}")

# upload the results to https://github.com/mendelsshop/git_function_history/stats/downloads.json
# with this format: {"schemaVersion":1,"label":"Crates.io Total Downloads","message":"0","color":"black"}
base64_json = {"schemaVersion":1,"label":"Crates.io Total Downloads","message":f"{count}","color":"black"}
base64_json = json.dumps(base64_json)

# using an access token
g = Github(sys.argv[1])

# get last sha
sha = g.get_repo("mendelsshop/git_function_history").get_contents("stats/downloads.json").sha

# update the file
g.get_repo("mendelsshop/git_function_history").update_file("stats/downloads.json", "update downloads.json", base64_json, sha)