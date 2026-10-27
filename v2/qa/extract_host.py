import pathlib,hashlib,json
r=pathlib.Path(__file__).resolve().parent;out=r/'extracted';out.mkdir(exist_ok=True);records=[]
def extract(path,signature,name):
 text=(r/'fixtures/host'/path).read_text();start=text.index(signature);body=text.index('{',start);n=1;i=body+1
 while n:
  if text[i]=='{':n+=1
  elif text[i]=='}':n-=1
  i+=1
 b=text[start:i].encode();wrapped=(b'impl Worker {\n'+b+b'\n}\n') if name=='clipboard_handle.rs' else (b'impl Host {\n'+b+b'\n}\n') if name in ['request.rs','granted.rs','require.rs'] else b+b'\n';(out/name).write_bytes(wrapped);records.append(dict(source=path,signature=signature,sha256=hashlib.sha256(b).hexdigest(),startByte=len(text[:start].encode()),bytes=len(b)))
for sig,name in [('pub async fn request(', 'request.rs'),('fn granted(', 'granted.rs'),('fn require(', 'require.rs'),("fn string<'a>", 'string.rs')]:extract('src-tauri/src/native_core/extensions/process_host.rs',sig,name)
extract('src-tauri/src/native_core/launcher/clipboard.rs','fn handle(&mut self,','clipboard_handle.rs')
(out/'provenance.json').write_text(json.dumps(records,indent=2)+'\n')
