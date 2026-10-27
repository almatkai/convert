#!/usr/bin/env python3
"""Real SDK JSONL child. OS reverse calls are captured, never delivered."""
import json,subprocess,pathlib,sys,queue,threading,tempfile,time,hashlib
ROOT=pathlib.Path(__file__).resolve().parents[2]; WORK=ROOT/'v2/.work';WORK.mkdir(exist_ok=True)
binary=pathlib.Path(sys.argv[1]).resolve(); counts={'views':0,'copy_bytes':0,'reverse_calls':0,'churn':0}
class Child:
 def __init__(self):
  self.p=subprocess.Popen([str(binary)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,env={**__import__('os').environ,'HOME':str(WORK/'home'),'TMPDIR':str(WORK/'tmp')});self.q=queue.Queue();self.n=0
  def reader():
   for line in self.p.stdout:self.q.put(json.loads(line))
   self.q.put(None)
  threading.Thread(target=reader,daemon=True).start();hello=self.get();assert hello['type']=='hello' and hello['id']=='com.tezbar.native.convert'
  self.rpc('initialize',dict(protocol=1,extensionId='com.tezbar.native.convert',platform='macOS',packageDir=str(binary.parent),dataDir=str(WORK/'data'),tempDir=str(WORK/'tmp'),grantedCapabilities=['clipboard.write']))
 def get(self,timeout=10):
  x=self.q.get(timeout=timeout);assert x is not None,'unexpected EOF';return x
 def send(self,method,params):
  self.n+=1;i=f'h:{__import__("uuid").uuid4()}';self.p.stdin.write(json.dumps(dict(type='request',id=i,method=method,params=params))+'\n');self.p.stdin.flush();return i
 def response(self,i):
  x=self.get();assert x.get('id')==i and x['type']=='response',x;return x
 def rpc(self,method,params):
  x=self.response(self.send(method,params));assert 'error' not in x,x;return x['result']
 def call(self,s,op,p=None):return self.rpc('session.call',dict(sessionId=s,operation='extension:'+op,payload=p or {}))
 def open(self,s,p=None):return self.rpc('session.open',dict(sessionId=s,commandName='convert',preferences=p or {},argumentValues={}))
 def search(self,s,q,rev=None):return self.call(s,'search-text-changed',dict(searchText=q,**({}if rev is None else {'clientInputRevision':rev})))
 def dispose(self,s):self.rpc('session.dispose',dict(sessionId=s))
 def reply(self,req,result=None,error=None):
  self.p.stdin.write(json.dumps(dict(type='response',id=req['id'],**({'error':error}if error else {'result':result})))+'\n');self.p.stdin.flush()
 def stop(self):self.rpc('shutdown',{});self.p.stdin.close();assert self.p.wait(timeout=5)==0,self.p.stderr.read()
def rows(v):return v['root']['children'][0]['children']
def token(v,row=0):return rows(v)[row]['props']['actionIds'][0]
def inspect(v,case):
 counts['views']+=1
 got=[dict(title=r['props']['title'],subtitle=r['props']['subtitle'],accessory=r['props']['accessories'][0]['text'],swatch=(r['props'].get('icon')or{}).get('tintColor',{}).get('light'))for r in rows(v)]
 want=[{k:r[k] for k in ['title','subtitle','accessory','swatch']}for r in case['rows']];assert got==want,(case['query'],got,want)
 assert v['root']['props']['__hasServerSearch'] is True
 for r in rows(v):assert set(r['props']['actionIds'])<=set(a['id']for a in v['actions'])
c=Child();assert not rows(c.open('main'))
# A crossed OLD client input must not revoke the newest model or its tokens.
c.open('revision-floor')
newest=c.search('revision-floor','32px',2);new_token=token(newest)
assert c.search('revision-floor','16px',1)['mode']=='unchanged'
same=c.call('revision-floor','refresh-session');assert same['root']['props']['searchText']=='32px' and token(same)==new_token
assert c.search('revision-floor','32px',2)['mode']=='unchanged'
# New malformed input owns the floor BEFORE text validation; an older valid
# input cannot resurrect its rows, even when the new draft has no result.
invalid=c.search('revision-floor','x'*4097,3);assert not invalid['actions']
assert c.search('revision-floor','16px',2)['mode']=='unchanged'
assert c.call('revision-floor','invoke-action',{'actionId':new_token})['mode']=='unchanged'
valid=c.search('revision-floor','48px',4);assert valid['actions'] and valid['root']['props']['clientInputRevision']==4
c.dispose('revision-floor');counts['revision_floor_checks']=7
corpus=json.loads((ROOT/'v2/qa/oracle.json').read_text());pref=None;sid='main';rev=0
# Reopen only when the source's module-scope preference snapshot changes.
for i,case in enumerate(corpus):
 p=case['preferences']
 if p!=pref:
  c.dispose(sid);sid=f'parity-{i}';assert not rows(c.open(sid,p));pref=p;rev=0
 rev+=1;v=c.search(sid,case['query'],rev);inspect(v,case)
 # Execute every row action for fixed edge/data cases and one out of 47 random cases.
 if i<250 or i%47==0:
  for j,r in enumerate(case['rows']):
   aid=token(v,j);reqid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':aid}))
   req=c.get();assert req['method']=='host.clipboard.write' and req['params']=={'text':r['copy']},req;counts['copy_bytes']+=len(r['copy'].encode());counts['reverse_calls']+=1
   c.reply(req,{'ok':True});v=c.response(reqid)['result'];assert v['message']=='Copied to clipboard'
   assert c.call(sid,'invoke-action',{'actionId':aid})['mode']=='unchanged'
# Malformed/oversized input must revoke. A/B/A receives distinct ownership.
v=c.search(sid,'16px',max(rev+1,10000));old=token(v)
for bad in [None,123,[],{},'x'*4097]:
 v=c.search(sid,bad);assert not v['actions'];assert c.call(sid,'invoke-action',{'actionId':old})['mode']=='unchanged'
for text in ['16px','32px','16px','bad','16px']:
 v=c.search(sid,text);assert v['root']['props']['searchText']==text
# Unknown ACK closed-shape validation, exact retry retained in selected row.
for ack in [None,{'ok':False},{'ok':True,'unexpected':1},{'ok':True,'message':None}]:
 v=c.search(sid,'#fafafa');row=3;original=token(v,row)
 reqid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':original}));req=c.get();frozen=req['params'];c.reply(req,ack);v=c.response(reqid)['result'];assert 'unknown' in v['message']
 retry=token(v,row);assert retry!=original and len(v['actions'])==1 and v['actions'][0]['id'] in rows(v)[row]['props']['actionIds']
 rid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':retry}));req=c.get();assert req['params']==frozen;c.reply(req,{'ok':True,'message':'Copied'});v=c.response(rid)['result'];assert len(v['actions'])==len(rows(v))
# Preference bounds are explicit and remain revoked on subsequent searches.
bounded=c.open('pref-bounds',{'basePixel':'0'*1025});assert '1024' in bounded['message'] and not bounded['actions']
bounded=c.search('pref-bounds','1rem');assert '1024' in bounded['message'] and not bounded['actions'];c.dispose('pref-bounds')
# Foreign token inert.
c.open('foreign');assert c.call('foreign','invoke-action',{'actionId':token(v)})['mode']=='unchanged';c.dispose('foreign')
# Cancel after reverse request: token cannot resurrect from SDK's staged rollback.
v=c.search(sid,'22px');aid=token(v);rid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':aid}));req=c.get();frozen=req['params']
c.p.stdin.write(json.dumps(dict(type='cancel',requestId=rid,reason='QA'))+'\n');c.p.stdin.flush()
seen=[]
while True:
 x=c.get();seen.append(x)
 if x.get('id')==rid:assert 'error'in x;break
assert c.call(sid,'invoke-action',{'actionId':aid})['mode']=='unchanged'
v=c.call(sid,'refresh-session');assert 'unknown'in v['message'];retry=token(v)
rid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':retry}));req=c.get();assert req['params']==frozen;c.reply(req,{'ok':True});c.response(rid)
# Timeout: SDK reverse waiter cancelled, explicit retry retained.
v=c.search(sid,'23px');rid=c.send('session.call',dict(sessionId=sid,operation='extension:invoke-action',payload={'actionId':token(v)}));req=c.get();pending=c.get(timeout=7)
if pending['type']=='cancel':pending=c.get()
assert pending['id']==rid and 'unknown'in pending['result']['message']
# Actual reader-order latest-search and late-ACK generation fences.
ids=[]
for i in range(40):ids.append(c.send('session.call',dict(sessionId=sid,operation='extension:search-text-changed',payload={'searchText':f'{i+1}px','clientInputRevision':20000+i})))
answers={}
while len(answers)<len(ids):
 x=c.get();assert x['type']=='response' and x['id']in ids,x;answers[x['id']]=x
assert answers[ids[-1]]['result']['root']['props']['searchText']=='40px'
assert c.call(sid,'refresh-session')['root']['props']['searchText']=='40px'
c.open('generation');old=c.search('generation','16px');oldtoken=token(old)
rid=c.send('session.call',dict(sessionId='generation',operation='extension:invoke-action',payload={'actionId':oldtoken}));reverse=c.get();did=c.send('session.dispose',{'sessionId':'generation'});done=set()
while len(done)<2:
 x=c.get()
 if x['type']=='cancel':continue
 assert x['id']in [rid,did],x;done.add(x['id'])
c.open('generation');new=c.search('generation','32px');c.reply(reverse,{'ok':True})
assert c.call('generation','invoke-action',{'actionId':oldtoken})['mode']=='unchanged'
assert c.call('generation','refresh-session')['root']['props']['searchText']=='32px'
c.dispose('generation')
c.dispose(sid)
# Cancelled opens and returned scope guards, then 160+ churn and real live capacity.
for i in range(180):
 s=f'churn-{i}';c.open(s);c.dispose(s);counts['churn']+=1
for i in range(128):c.open(f'live-{i}')
overflow=c.response(c.send('session.open',dict(sessionId='overflow',commandName='convert',preferences={},argumentValues={})));assert 'error' in overflow
assert c.search('live-127','12pt')['actions']
for i in range(128):c.dispose(f'live-{i}')
for i in range(12):
 s=f'cancel-open-{i}';rid=c.send('session.open',dict(sessionId=s,commandName='convert',preferences={},argumentValues={}));c.p.stdin.write(json.dumps(dict(type='cancel',requestId=rid))+'\n');c.p.stdin.flush();c.response(rid)
 # Either open completed, or runner removed the failed open.
 x=c.response(c.send('session.dispose',{'sessionId':s}));assert 'result'in x or 'error'in x
c.open('reused');c.dispose('reused');c.open('reused');assert c.search('reused','1rem')['actions'];c.dispose('reused')
c.stop()
# EOF while reverse request pending must exit, no OS calls and no detached work.
c=Child();c.open('eof');v=c.search('eof','1rem');c.send('session.call',dict(sessionId='eof',operation='extension:invoke-action',payload={'actionId':token(v)}));assert c.get()['method']=='host.clipboard.write';c.p.stdin.close();assert c.p.wait(timeout=5)==0
print(json.dumps(counts,sort_keys=True));(WORK/'process-result.json').write_text(json.dumps(counts,indent=2)+'\n')
