#!/usr/bin/env python3
"""Reproducible isolated QA; never touches host checkout/profile or real clipboard."""
import pathlib,os,sys,subprocess,json,shutil,hashlib,time
r=pathlib.Path(__file__).resolve().parents[2];v=r/'v2';w=v/'.work';w.mkdir(exist_ok=True)
for n in ['home','tmp','npm','data','xdg-cache','xdg-config','xdg-data','cargo-dev','target-dev']:(w/n).mkdir(exist_ok=True)
env={**os.environ,'HOME':str(w/'home'),'TMPDIR':str(w/'tmp'),'TMP':str(w/'tmp'),'TEMP':str(w/'tmp'),'XDG_CACHE_HOME':str(w/'xdg-cache'),'XDG_CONFIG_HOME':str(w/'xdg-config'),'XDG_DATA_HOME':str(w/'xdg-data'),'npm_config_cache':str(w/'npm'),'npm_config_update_notifier':'false','CARGO_HOME':str(w/'cargo-dev'),'CARGO_TARGET_DIR':str(w/'target-dev'),'RUSTUP_HOME':os.environ.get('RUSTUP_HOME','/Users/almat/.rustup')}
results=json.loads((v/'qa/validation-results.json').read_text()) if '--resume' in sys.argv and (v/'qa/validation-results.json').exists() else []
def run(name,args,extra=None,expected_failure=False):
 if '--resume' in sys.argv and any(x['name']==name and x['exit']==0 for x in results):
  print(name+' (previous pass retained)',flush=True);return
 results[:]=[x for x in results if x['name']!=name]
 print(name,flush=True);start=time.monotonic();p=subprocess.run(args,cwd=r,env={**env,**(extra or {})},stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True);(w/(name+'.log')).write_text(p.stdout)
 item=dict(name=name,exit=p.returncode,seconds=round(time.monotonic()-start,3),logSha256=hashlib.sha256(p.stdout.encode()).hexdigest());results.append(item);print(p.stdout[-1800:],flush=True)
 if p.returncode and not expected_failure:raise RuntimeError(name+' failed')
 return p
try:
 assert subprocess.check_output(['git','rev-parse','--show-toplevel'],cwd=r,text=True).strip()==str(r)
 for name,source in [('oracle',v/'qa/fixtures/v1'),('schema-tools',v/'qa/schema-tools')]:
  dest=w/name
  if not dest.exists():shutil.copytree(source,dest)
  run('npm-'+name,['npm','ci','--prefix',str(dest),'--ignore-scripts','--no-audit','--no-fund','--fetch-timeout=30000','--fetch-retries=1'])
 run('v1-typecheck',['npm','--prefix',str(w/'oracle'),'run','typecheck'])
 run('v1-test',['npm','--prefix',str(w/'oracle'),'test'])
 run('oracle',['node',str(v/'qa/oracle.cjs'),str(v/'qa/cases.json'),str(w/'oracle-recomputed.json')])
 assert (w/'oracle-recomputed.json').read_bytes()==(v/'qa/oracle.json').read_bytes(),'Source oracle drift'
 run('producer',['node',str(v/'qa/producer.cjs')]);run('renderer',['node',str(v/'qa/renderer.cjs')]);run('host-type-context',['node',str(v/'qa/typecheck.cjs')]);run('schema',['node',str(v/'qa/schema.cjs')])
 run('extract-host',[sys.executable,str(v/'qa/extract_host.py')])
 run('cargo-fetch',['cargo','+stable','fetch','--locked','--manifest-path',str(v/'Cargo.toml')])
 run('rustfmt',['cargo','+stable','fmt','--check','--manifest-path',str(v/'Cargo.toml')])
 for tool,target in [('stable','target-dev'),('1.85.0','target-msrv')]:
  for op in ['test','clippy']:
   cmd=['cargo','+'+tool,op,'--offline','--locked','--all-targets','--manifest-path',str(v/'Cargo.toml')]
   if op=='clippy':cmd+=['--','-D','warnings']
   run(tool+'-'+op,cmd,{'CARGO_TARGET_DIR':str(w/target)})
 for op in ['check','clippy','build']:
  cmd=['cargo','+stable',op,'--offline','--locked','--all-targets','--target','x86_64-pc-windows-gnu','--manifest-path',str(v/'Cargo.toml')]
  if op=='clippy':cmd+=['--','-D','warnings']
  run('windows-gnu-'+op,cmd,{'CARGO_TARGET_DIR':str(w/'target-win')},expected_failure=True)
 run('debug-build',['cargo','+stable','build','--offline','--locked','--manifest-path',str(v/'Cargo.toml')])
 run('process-debug',[sys.executable,str(v/'qa/process.py'),str(w/'target-dev/debug/tezbar-convert')])
 run('archives-and-grants',[sys.executable,str(v/'qa/audit.py')])
 if '--fresh-release'in sys.argv:
  run('fresh-release',[sys.executable,str(v/'qa/build_fresh.py')]);run('dev-zip',[sys.executable,str(v/'qa/package.py')]);run('extracted-process',[sys.executable,str(v/'qa/process.py'),str(w/'dev-extracted/bin/tezbar-convert')])
finally:
 (v/'qa/validation-results.json').write_text(json.dumps(results,indent=2)+'\n')
