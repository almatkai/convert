#!/usr/bin/env python3
"""Git-visible worktree byte inventory; parent must rehash staged blobs independently."""
import pathlib,subprocess,json,hashlib,stat,sys
r=pathlib.Path(__file__).resolve().parents[2];dest=r/'v2/INVENTORY.json'
paths=subprocess.check_output(['git','ls-files','-z','--cached','--others','--exclude-standard','--','README.md','v2'],cwd=r).decode().split('\0')
entries=[]
for p in sorted(set(filter(None,paths))):
 f=r/p
 if f==dest:continue
 assert f.is_file() and not f.is_symlink(),p
 b=f.read_bytes();entries.append(dict(path=p,sha256=hashlib.sha256(b).hexdigest(),size=len(b),mode=f'{stat.S_IMODE(f.stat().st_mode):04o}'))
body=(json.dumps(dict(kind='worktree-bytes-not-staged-blobs',selfExcluded='v2/INVENTORY.json',files=entries),indent=2)+'\n').encode()
if '--check' in sys.argv:assert dest.read_bytes()==body,'Inventory drift'
else:dest.write_bytes(body)
print(f'Inventory: {len(entries)} files; sha256 {hashlib.sha256(body).hexdigest()}')
