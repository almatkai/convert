#!/usr/bin/env python3
import pathlib,hashlib,json,zipfile,subprocess,stat,sys
r=pathlib.Path(__file__).resolve().parents[2];v=r/'v2';w=v/'.work'
def sha(b):return hashlib.sha256(b).hexdigest()
# Fresh immutable SDK fetched over HTTPS and exact product source binding.
bind=json.loads((w/'release-bind-before.json').read_text());assert all(sha((r/p).read_bytes())==h for p,h in bind.items())
release=json.loads((w/'current-release.json').read_text()) if (w/'current-release.json').exists() else dict(cargoHome=str(w/'cargo-release'),target=str(w/'target-release'),freshEmpty=True)
sdkhomes=list((pathlib.Path(release['cargoHome'])/'git/checkouts').glob('*/e955911'));assert len(sdkhomes)==1,sdkhomes
sdk=sdkhomes[0];pin='e955911f0bf5b558f9d3eb5183df0adc95c22af2';assert subprocess.check_output(['git','-C',str(sdk),'rev-parse','HEAD'],text=True).strip()==pin
sdkfiles=subprocess.check_output(['git','-C',str(sdk),'ls-tree','-r','--name-only',pin],text=True).splitlines();sdkhash={}
for p in sdkfiles:
 expected=subprocess.check_output(['git','-C',str(sdk),'show',pin+':'+p]);assert (sdk/p).read_bytes()==expected;sdkhash[p]=sha(expected)
binpath=pathlib.Path(release['target'])/'aarch64-apple-darwin/release/tezbar-convert';binary=binpath.read_bytes();info=json.loads(subprocess.check_output([str(binpath),'--protocol-info']));assert info['commands']==['convert'] and info['sdkVersion']=='0.1.1'
assert info==json.loads(subprocess.check_output([str(binpath),'--self-test']))
members={'bin/tezbar-convert':(binary,0o755),'resources/icon.png':((v/'assets/icon.png').read_bytes(),0o644),'LICENSE':((v/'LICENSE').read_bytes(),0o644),'NOTICE':((v/'NOTICE').read_bytes(),0o644)}
for folder in ['cargo','sdk','raycast']:
 for p in (v/'notices'/folder).rglob('*'):
  if p.is_file():members[str(p.relative_to(v))]=(p.read_bytes(),0o644)
members['notices/own-v1-LICENSE']=((v/'notices/own-v1-LICENSE').read_bytes(),0o644)
manifest=json.loads((v/'extension.json').read_text());manifest['files']={p:sha(b)for p,(b,_)in sorted(members.items())};members['extension.json']=((json.dumps(manifest,indent=2)+'\n').encode(),0o644)
archive=w/'convert-aarch64-apple-darwin-DEV.zip'
def write_zip(dest):
 with zipfile.ZipFile(dest,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9)as z:
  for name,(data,mode)in sorted(members.items()):
   i=zipfile.ZipInfo(name,(1980,1,1,0,0,0));i.create_system=3;i.external_attr=(stat.S_IFREG|mode)<<16;i.compress_type=zipfile.ZIP_DEFLATED;z.writestr(i,data)
write_zip(archive);again=w/'dev-repeat.zip';write_zip(again);assert archive.read_bytes()==again.read_bytes()
extracted=w/'dev-extracted';extracted.mkdir(exist_ok=True)
with zipfile.ZipFile(archive)as z:
 assert set(z.namelist())==set(manifest['files'])|{'extension.json'}
 for i in z.infolist():
  assert i.filename in members;data=z.read(i);expected,mode=members[i.filename];assert data==expected and stat.S_IMODE(i.external_attr>>16)==mode
  p=extracted/i.filename;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(data);p.chmod(mode)
 assert all(sha(z.read(p))==h for p,h in manifest['files'].items())
exe=extracted/'bin/tezbar-convert';assert json.loads(subprocess.check_output([str(exe),'--protocol-info']))==info
result=dict(archive=str(archive.relative_to(r)),sha256=sha(archive.read_bytes()),size=archive.stat().st_size,executableSha256=sha(binary),protocolInfo=info,members=[dict(path=p,sha256=sha(b),size=len(b),mode=oct(mode))for p,(b,mode)in sorted(members.items())],sourceBinding=bind,freshBuild=release,sdkGit=pin,sdkFiles=sdkhash,compiler=subprocess.check_output(['rustc','+stable','-Vv'],text=True),deterministicZipSameInput=True,compilerOutputReproducibilityClaim=False)
(v/'qa/dev-artifact.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:result[k]for k in ['archive','sha256','executableSha256','size']}));print('members',len(members))
