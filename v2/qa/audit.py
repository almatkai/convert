#!/usr/bin/env python3
"""Audit exact Cargo registry archives and executed npm tools; retain authentic grants."""
import pathlib,hashlib,json,tomllib,tarfile,io,base64,urllib.request,os
ROOT=pathlib.Path(__file__).resolve().parents[2];v=ROOT/'v2';w=v/'.work';notices=v/'notices';notices.mkdir(exist_ok=True)
def sha(b):return hashlib.sha256(b).hexdigest()
def grants(tar):
 result=[]
 for m in tar.getmembers():
  name=pathlib.PurePosixPath(m.name)
  if m.isfile() and len(name.parts)<=3 and any(s in name.name.lower() for s in ['license','licence','copying','copyright','authors','notice']):
   b=tar.extractfile(m).read();result.append((m.name,b))
 return result
cargo=[]
for p in tomllib.loads((v/'Cargo.lock').read_text())['package']:
 if not p.get('source','').startswith('registry+'):continue
 files=list((w/'cargo-dev/registry/cache').glob(f'*/{p["name"]}-{p["version"]}.crate'));assert len(files)==1,p
 b=files[0].read_bytes();assert sha(b)==p['checksum'],p
 tar=tarfile.open(fileobj=io.BytesIO(b),mode='r:gz'); gs=grants(tar); out=notices/'cargo'/f'{p["name"]}-{p["version"]}';out.mkdir(parents=True,exist_ok=True)
 records=[]
 for name,data in gs:
  o=out/pathlib.PurePosixPath(name).relative_to(f'{p["name"]}-{p["version"]}');o.parent.mkdir(parents=True,exist_ok=True);o.write_bytes(data);records.append(dict(path=str(o.relative_to(v)),upstream=name,sha256=sha(data),size=len(data)))
 # Expanded registry sources must match archive content, including AUTHORS.
 expanded=list((w/'cargo-dev/registry/src').glob(f'*/{p["name"]}-{p["version"]}'));assert len(expanded)==1
 for m in tar.getmembers():
  if m.isfile():assert (expanded[0]/pathlib.PurePosixPath(m.name).relative_to(expanded[0].name)).read_bytes()==tar.extractfile(m).read(),m.name
 meta=tomllib.loads(tar.extractfile(f'{p["name"]}-{p["version"]}/Cargo.toml').read().decode())['package']
 cargo.append(dict(name=p['name'],version=p['version'],archiveSha256=sha(b),checksum=p['checksum'],license=meta.get('license'),licenseFile=meta.get('license-file'),grants=records))
(v/'qa/cargo-audit.json').write_text(json.dumps(cargo,indent=2)+'\n')
npm=[]
entries=[]
for tool,lockpath in [('oracle',v/'qa/fixtures/v1/package-lock.json'),('schema-tools',v/'qa/schema-tools/package-lock.json')]:
 for name,p in json.loads(lockpath.read_text())['packages'].items():
  entries.append((tool,name,p))
for tool,name,p in entries:
 if not name or not (w/tool/name).exists():continue
 sri=p['integrity'];algo,encoded=sri.split('-',1);digest=base64.b64decode(encoded).hex();cache=w/'npm/_cacache/content-v2'/algo/digest[:2]/digest[2:4]/digest[4:]
 if cache.exists():b=cache.read_bytes()
 else:b=urllib.request.urlopen(p['resolved'],timeout=30).read()
 assert base64.b64encode(hashlib.new(algo,b).digest()).decode()==encoded,p
 t=tarfile.open(fileobj=io.BytesIO(b),mode='r:gz');meta_member=min((m for m in t.getmembers() if m.isfile() and m.name.endswith('/package.json')),key=lambda m:len(pathlib.PurePosixPath(m.name).parts));packroot=pathlib.PurePosixPath(meta_member.name).parent;meta=json.load(t.extractfile(meta_member));gs=grants(t);out=notices/'npm'/(meta['name'].replace('/','__')+'-'+meta['version']);out.mkdir(parents=True,exist_ok=True);records=[];code=hashlib.sha256();members=0
 for m in t.getmembers():
  if m.isfile():
   data=t.extractfile(m).read();code.update(m.name.encode()+b'\0'+bytes.fromhex(sha(data)));members+=1
   # npm may adjust package metadata or executable flags, never JS/TS source bytes.
   rel=pathlib.PurePosixPath(m.name).relative_to(packroot);installed=w/tool/name/rel
   if installed.exists() and installed.suffix in ['.js','.cjs','.mjs','.ts','.json'] and rel.name!='package.json':assert installed.read_bytes()==data,str(installed)
 for gn,data in gs:
  o=out/pathlib.PurePosixPath(gn).relative_to(packroot);o.parent.mkdir(parents=True,exist_ok=True);o.write_bytes(data);records.append(dict(path=str(o.relative_to(v)),upstream=gn,sha256=sha(data),size=len(data)))
 npm.append(dict(tool=tool,name=meta['name'],version=meta['version'],url=p['resolved'],integrity=sri,archiveSha256=sha(b),members=members,contentInventorySha256=code.hexdigest(),license=meta.get('license'),grants=records))
for item in npm:
 if item['name'].startswith('@esbuild/') and not item['grants']:
  parent=next(p for p in npm if p['name']=='esbuild' and p['version']==item['version'])
  item['matchingParentGrantFiles']=parent['grants']
(v/'qa/npm-audit.json').write_text(json.dumps(npm,indent=2)+'\n')
print(f'Cargo: {len(cargo)} exact registry archives; npm: {len(npm)} installed locked tarballs')
print('Packages without retained grant files:',[p['name'] for p in cargo+npm if not p['grants']])
