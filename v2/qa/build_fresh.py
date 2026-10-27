#!/usr/bin/env python3
import pathlib,os,json,hashlib,subprocess,uuid
r=pathlib.Path(__file__).resolve().parents[2];v=r/'v2';w=v/'.work';tag=uuid.uuid4().hex
home=w/f'cargo-release-{tag}';target=w/f'target-release-{tag}';assert not home.exists() and not target.exists()
files=[p for p in v.rglob('*') if p.is_file() and (p.is_relative_to(v/'src') or p.name in ['Cargo.toml','Cargo.lock','extension.json'] and p.parent==v)]
bind={str(p.relative_to(r)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files};(w/'release-bind-before.json').write_text(json.dumps(bind,indent=2)+'\n')
env={**os.environ,'HOME':str(w/'home'),'CARGO_HOME':str(home),'CARGO_TARGET_DIR':str(target),'RUSTUP_HOME':os.environ.get('RUSTUP_HOME','/Users/almat/.rustup'),'TMPDIR':str(w/'tmp')}
subprocess.run(['cargo','+stable','build','--locked','--release','--target','aarch64-apple-darwin','--manifest-path',str(v/'Cargo.toml')],env=env,check=True)
assert all(hashlib.sha256((r/p).read_bytes()).hexdigest()==h for p,h in bind.items())
(w/'current-release.json').write_text(json.dumps(dict(cargoHome=str(home),target=str(target),freshEmpty=True),indent=2)+'\n')
