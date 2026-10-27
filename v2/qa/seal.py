#!/usr/bin/env python3
import pathlib,subprocess,json,hashlib
r=pathlib.Path(__file__).resolve().parents[2];v=r/'v2';pin='92e37e66e3fbeff22fbf9374c97af36a91832cd6'
def git(*a):return subprocess.check_output(['git',*a],cwd=r)
def sha(b):return hashlib.sha256(b).hexdigest()
assert git('rev-parse','--show-toplevel').decode().strip()==str(r)
assert git('rev-parse','HEAD').decode().strip()==pin
assert git('branch','--show-current').decode().strip()=='feat/native-v2-rust'
assert not git('diff','--cached','--name-only')
v1=[]
for p in git('ls-tree','-r','--name-only',pin).decode().splitlines():
 expected=git('show',pin+':'+p);actual=(r/p).read_bytes()
 if p=='README.md':assert actual==expected+b'\nStandalone Rust V2: [implementation and validation](v2/README.md).\n'
 else:assert actual==expected,p
 v1.append(dict(path=p,baselineSha256=sha(expected),worktreeSha256=sha(actual)))
records=json.loads((v/'qa/provenance.json').read_text())
for item in records:
 p=v/'qa/fixtures'/item['kind']/item['path'];b=p.read_bytes();assert sha(b)==item['sha256'] and len(b)==item['size'],p
for item in json.loads((v/'qa/fixtures/raycast/fetch.json').read_text()):
 if item['status']==200:assert sha((v/'qa/fixtures/raycast'/item['path']).read_bytes())==item['sha256']
icon=(r/'assets/icon.png').read_bytes();assert icon==(v/'assets/icon.png').read_bytes()==(v/'qa/fixtures/raycast/extensions/convert/assets/icon.png').read_bytes()
m=json.loads((v/'extension.json').read_text());assert m['artifacts']=={} and all(x['state']=='not-run'for x in m['verification'])
for p in git('ls-files','--others','--exclude-standard').decode().splitlines():assert p.startswith('v2/'),p
for p in git('diff','--name-only').decode().splitlines():assert p=='README.md' or p.startswith('v2/'),p
out=dict(head=pin,branch='feat/native-v2-rust',emptyIndex=True,iconSha256=sha(icon),verifiedGitFixtures=len(records),v1=v1)
(v/'qa/fixture-seal.json').write_text(json.dumps(out,indent=2)+'\n');print(f'Seal: {len(v1)} original tracked files, {len(records)} Git fixtures, icon exact, empty index, permitted paths only')
