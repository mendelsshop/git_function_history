import json
import sys
from github import Github
from subprocess import Popen, PIPE
import os
if len(sys.argv) != 2:
    print(f"Usage: {sys.argv[0]} <github token>")
    sys.exit(1)

os.system("git switch main")

t = Popen(["tokei", "--output=json"], stdout=PIPE, stderr=PIPE)

stdout, stderr = t.communicate()
if stderr:
    print(f"Error: {stderr}")
    sys.exit(1)

t = json.loads(stdout),
code = int(t[0]['Total']['code'])
blanks = int(t[0]['Total']['blanks'])
comments = int(t[0]['Total']['comments'])
count = code + blanks + comments

print(f"Total: {count}")

# upload the results to github
# with this format: {"schemaVersion":1,"label":"Crates.io Total Downloads","message":"0","color":"black"}
base64_json = {"schemaVersion":1,"label":"Total Lines of Code","message":f"{count}","color":"black"}
base64_json = json.dumps(base64_json)

# using an access token
g = Github(sys.argv[1])

# get last sha
git = g.get_repo("mendelsshop/git_function_history")
commit = git.get_contents("loc.json", ref="stats")
old = commit.decoded_content
if old == base64_json:
    print("smae")
os.system("git switch stats")
exit()
# update the file
git.update_file("loc.json", "update loc.json", base64_json, commit.sha)

