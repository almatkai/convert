// Full immutable/proposed TSX modules, lightweight deterministic hook scheduler.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const ts=require('../.work/oracle/node_modules/typescript');
const base=path.join(__dirname,'fixtures/host'),proposed=path.join(__dirname,'../integration/proposed');
const jsx=(type,props)=>({type,props:props||{}});const UI={TextField:'TextField',Button:'Button',Message:'Message',Hint:'Hint',HintBar:'HintBar',Kbd:'Kbd',ViewHeader:'ViewHeader',cx:(...s)=>s.join(' ')};
function scheduler(){let slots=[],i=0,effects=[];const h={
 useState(initial){const n=i++;if(!slots[n])slots[n]={v:typeof initial==='function'?initial():initial};return[slots[n].v,v=>{slots[n].v=typeof v==='function'?v(slots[n].v):v}];},
 useReducer(reducer,initial){const [state,set]=h.useState(initial);return [state,action=>set(old=>reducer(old,action))];},
 useRef(initial){const n=i++;if(!slots[n])slots[n]={current:initial};return slots[n];},
 useMemo(fn,deps){const n=i++;if(!slots[n]||!same(slots[n].deps,deps))slots[n]={v:fn(),deps};return slots[n].v;},
 useCallback(fn,deps){return h.useMemo(()=>fn,deps);},
 useEffect(fn,deps){const n=i++;if(!slots[n]||!same(slots[n].deps,deps)){effects.push(()=>{slots[n]?.clean?.();slots[n]={deps,clean:fn()}})}},
 createElement:(t,p,...c)=>jsx(t,{...p,children:c.length===1?c[0]:c}),
 };return{h,render(fn,p){i=0;const x=fn(p);for(const e of effects.splice(0))e();return x;},dispose(){for(const s of slots)s?.clean?.();slots=[];}};}
function same(a,b){return a&&b&&a.length===b.length&&a.every((v,i)=>v===b[i]);}
function load(file,h,overrides={}){
 const exports={};const req=id=>{
  if(id==='react')return {...h,default:h};if(id==='react/jsx-runtime')return{jsx,jsxs:jsx,Fragment:'Fragment'};
  if(id in overrides)return overrides[id];if(id.includes('primitives'))return UI;
  if(id==='./extensionPreferences')return load(path.join(base,'src/renderer/extensionPreferences.ts'),h);
  if(id==='./runtimeInputOwnership')return load(path.join(proposed,'src/renderer/runtimeInputOwnership.ts'),h);
  if(id.includes('extensionSessionRefresh'))return{startSessionRefresh:()=>()=>{}};
  if(id==='./src/raycast-api')return{ExtensionRuntimeSurface:'Surface'};
  if(id==='./action-runtime')return{ActionPanelOverlay:'Overlay',Action:{},ActionPanel:{}};
  if(id==='./action-runtime-registry')return{ActionRegistryContext:{Provider:'Provider'}};
  if(id==='./hooks')return{NavigationContext:{Provider:'Provider'},setRuntimeContext:()=>{}};
  if(id==='./list-runtime')return{ListRuntime:'ListRuntime'};
  if(id==='./form-runtime')return{FormRuntime:'FormRuntime'};
  if(id==='./grid-runtime')return{GridRuntime:'GridRuntime'};
  if(id==='./detail-runtime')return{DetailRuntime:'DetailRuntime'};
  if(id.includes('Markdown'))return{Markdown:'Markdown'};
  if(id.includes('colorConverter'))return{buildColorConversionResults:()=>{throw Error('Native rows replaced by generic converter')}};
  throw Error(`Unmocked import ${id} in ${file}`);
 };
 const code=ts.transpileModule(fs.readFileSync(file,'utf8'),{compilerOptions:{jsx:ts.JsxEmit.ReactJSX,module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022}}).outputText;
 vm.runInNewContext(code,{exports,require:req,console:{log(){},error(){}},window:global.window,document:{visibilityState:'visible'},setTimeout,clearTimeout,TextEncoder,React:h,Promise,Map,Set},{filename:file});return exports;
}
function find(tree,type){if(!tree||typeof tree!=='object')return null;if(tree.type===type)return tree;for(const c of [].concat(tree.props?.children||[]).flat(4)){const n=find(c,type);if(n)return n;}return null;}
function deferred(){let resolve,reject;const p=new Promise((a,b)=>{resolve=a;reject=b});return{p,resolve,reject};}
const tick=()=>new Promise(r=>setImmediate(r));
(async()=>{
 let calls=[];global.window={setTimeout,clearTimeout,addEventListener(){},removeEventListener(){},tezbar:{extensionSearchTextChanged:p=>{const d=deferred();calls.push({...d,kind:'search',p});return d.p;},extensionInvokeAction:p=>{const d=deferred();calls.push({...d,kind:'action',p});return d.p;},extensionRefreshSession:p=>{const d=deferred();calls.push({...d,kind:'refresh',p});return d.p;},extensionDisposeSession:async()=>{},onExtensionSessionInvalidated:()=>()=>{}}};
 const view=(q='',rev=null,g='g',s='s',nr=1)=>({ok:true,mode:'view',sessionId:s,generation:g,extensionId:'com.tezbar.native.convert',commandName:'convert',title:'Convert Unit',refreshMs:null,root:{type:'List',props:{searchText:q,clientInputRevision:rev,nativeModelRevision:nr,nativeSessionGeneration:'n',__hasServerSearch:true},children:[{type:'List.Item',props:{title:q,actionIds:['copy']}}]},actions:[{id:'copy',title:'Copy'}]});
 let sch=scheduler(),component=load(path.join(proposed,'src/renderer/ExtensionRuntimeView.tsx'),sch.h).default,initial=view();const render=()=>sch.render(component,{initial,onBack(){}});render();let tree=render(),surface=find(tree,'Surface');
 // Synchronous ownership revokes before raw input validation and before any promise.
 const a=surface.props.onSearchTextChanged('A');tree=render();assert.equal(find(tree,'Surface').props.actions.length,0);
 const b=find(tree,'Surface').props.onSearchTextChanged('B');const aa=find(tree,'Surface').props.onSearchTextChanged('A');assert.deepEqual(calls.map(c=>c.p.clientInputRevision),[1,2,3]);
 calls[0].resolve(view('A',1));calls[1].resolve(view('B',2));await tick();assert.equal(find(render(),'Surface').props.actions.length,0);
 calls[2].resolve(view('A',3));await Promise.all([a,b,aa]);assert.equal(find(render(),'Surface').props.root.props.searchText,'A');assert.equal(find(render(),'Surface').props.actions.length,1);
 const bad=find(render(),'Surface').props.onSearchTextChanged(null);await bad;assert.equal(find(render(),'Surface').props.actions.length,0);
 const valid=find(render(),'Surface').props.onSearchTextChanged('C');calls.at(-1).resolve(view('C',5));await valid;assert.equal(find(render(),'Surface').props.actions.length,1);
 // Late ACK cannot restore A after D. Duplicate local action inert.
 let srf=find(render(),'Surface');const copy=srf.props.onInvokeAction('copy');const before=calls.length;srf.props.onInvokeAction('copy');assert.equal(calls.length,before);
 const d=find(render(),'Surface').props.onSearchTextChanged('D');calls.at(-1).resolve(view('D',6));await d;calls.at(-2).resolve(view('C',5));await copy;assert.equal(find(render(),'Surface').props.root.props.searchText,'D');
 // Terminal false/unchanged/no-view/rejection releases only own pending state, never old Copy.
 for(const result of [{ok:false,message:'failed'},{ok:true,mode:'unchanged'},{ok:true,mode:'no-view',message:'none'},'reject']){
  const task=find(render(),'Surface').props.onSearchTextChanged('terminal');if(result==='reject')calls.at(-1).reject(new Error('failed'));else calls.at(-1).resolve(result);await task;assert.equal(find(render(),'Surface').props.actions.length,0);
 }
 const recovery=find(render(),'Surface').props.onSearchTextChanged('new');const rev=calls.at(-1).p.clientInputRevision;calls.at(-1).resolve(view('new',rev));await recovery;assert.equal(find(render(),'Surface').props.actions.length,1);
 // Native cannot manufacture revision, same/replaced generation and unmount fence late responses.
 const forged=find(render(),'Surface').props.onSearchTextChanged('forge');calls.at(-1).resolve(view('forge',100000));await forged;assert.equal(find(render(),'Surface').props.actions.length,0);
 const late=find(render(),'Surface').props.onSearchTextChanged('late');initial=view('replacement draft',null, 'replacement');render();calls.at(-1).resolve(view('late',rev+2));await late;assert.notEqual(find(render(),'Surface').props.root.props.searchText,'late');sch.dispose();
 // Execute pinned RuntimeView to establish the concrete ownership defect.
 sch=scheduler();const baseline=load(path.join(base,'src/renderer/ExtensionRuntimeView.tsx'),sch.h).default;
 const bi=view();const brender=()=>sch.render(baseline,{initial:bi,onBack(){}});brender();let bs=find(brender(),'Surface');const bp=bs.props.onSearchTextChanged('new draft');assert.equal(find(brender(),'Surface').props.actions.length,1);calls.at(-1).resolve(view('new draft',1));await bp;sch.dispose();
 // Execute pinned preference editor and its actual partition helpers; raw optional strings retained.
 const saved=[];window.tezbar.getExtensionPreferenceSetup=async p=>p.commandName?{preferences:[{name:'basePixel',title:'Base PX Value',type:'textfield',required:false,commandName:'convert'}],values:{basePixel:''}}:{preferences:[],values:{}};
 window.tezbar.saveExtensionPreferences=async p=>{saved.push(p);return true};
 sch=scheduler();const Editor=load(path.join(base,'src/renderer/ExtensionPreferencesEditor.tsx'),sch.h).ExtensionPreferencesEditor;const ep={extensionId:'com.tezbar.native.convert',extensionName:'Web Converter',commands:[{name:'convert'}],onMessage(){}};
 sch.render(Editor,ep);await tick();let ed=sch.render(Editor,ep);assert.equal(find(ed,'TextField').props.value,'');
 for(const value of ['0','-16','Infinity','bad','']){find(ed,'TextField').props.onChange({target:{value}});ed=sch.render(Editor,ep);find(ed,'Button').props.onClick();await tick();ed=sch.render(Editor,ep);assert.equal(saved.at(-1).commandName,'convert');assert.equal(saved.at(-1).values.basePixel,value);}
 sch.dispose();
 // Reusable controls keep complete drafts and an independent revision under A/B/A.
 sch=scheduler();component=load(path.join(proposed,'src/renderer/ExtensionRuntimeView.tsx'),sch.h).default;
 const controlled=(q='',rev=null)=>{const v=view(q,rev);v.root.props.searchBarAccessory={type:'List.Dropdown',props:{actionId:'ctl'}};v.actions.push({id:'ctl',kind:'action',title:'Base'});return v;};
 initial=controlled();const crender=()=>sch.render(component,{initial,onBack(){}});crender();let cf=find(crender(),'Surface');const start=calls.length;
 const ca=cf.props.onInvokeAction('ctl',{value:'A'});cf=find(crender(),'Surface');const cb=cf.props.onInvokeAction('ctl',{value:'B'});cf=find(crender(),'Surface');const caa=cf.props.onInvokeAction('ctl',{value:'A'});
 assert.equal(calls.length-start,3);assert.deepEqual(calls.slice(start).map(c=>c.p.formValues.clientInputRevision),[1,2,3]);assert.deepEqual(calls.slice(start).map(c=>c.p.formValues.controlValues.ctl),['A','B','A']);
 calls[start+1].resolve(controlled('',2));calls[start].resolve(controlled('',1));await tick();assert.equal(find(crender(),'Surface').props.actions.length,0);calls[start+2].resolve(controlled('',3));await Promise.all([ca,cb,caa]);assert(find(crender(),'Surface').props.actions.length);
 const qp=find(crender(),'Surface').props.onSearchTextChanged('pending');cf=find(crender(),'Surface');const cross=cf.props.onInvokeAction('ctl',{value:'new'});assert.equal(calls.at(-1).p.formValues.searchText,'pending');
 calls.at(-2).resolve(controlled('pending',4));calls.at(-1).resolve(controlled('',5));await Promise.all([qp,cross]);assert.equal(find(crender(),'Surface').props.actions.length,0);
 const corrected=find(crender(),'Surface').props.onInvokeAction('ctl',{value:'new'});calls.at(-1).resolve(controlled('pending',6));await corrected;assert(find(crender(),'Surface').props.actions.length);sch.dispose();
 // Full real Surface action filter, two IDs retained on the same row, not global-only.
 sch=scheduler();const Surface=load(path.join(base,'src/renderer/src/raycast-api/index.tsx'),sch.h).ExtensionRuntimeSurface;
 const props={sessionId:'s',title:'Convert Unit',commandName:'convert',extensionId:'native',root:view().root,actions:[{id:'first'},{id:'retry'},{id:'foreign'}],onBack(){},onInvokeAction(){},onSearchTextChanged(){},onLoadMore(){}};
 let x=sch.render(Surface,props);find(x,'ListRuntime').props.onOpenActions(['first','retry']);x=sch.render(Surface,props);assert.equal(JSON.stringify(find(x,'Overlay').props.actions.map(a=>a.id)),JSON.stringify(['first','retry']));sch.dispose();
 // Full proposed ListRuntime emits ownership at input-time without debounce; exact server rows kept.
 sch=scheduler();const List=load(path.join(proposed,'src/renderer/src/raycast-api/list-runtime.tsx'),sch.h).ListRuntime;let inputs=[];
 const lp={root:{type:'List',props:{__hasServerSearch:true,searchText:'',searchBarPlaceholder:'Type'},children:[]},title:'Convert Color',commandName:'convert',actions:[],onBack(){},onOpenActions(){},onRunPrimaryAction(){},onSearchTextChanged:q=>inputs.push(q),onLoadMore(){}};
 x=sch.render(List,lp);const input=find(x,'input');assert(input);for(const value of ['A','B','A'])input.props.onChange({target:{value}});assert.deepEqual(inputs,['A','B','A']);sch.dispose();
 // Retained callbacks must be inert BEFORE touching the replacement gate/model
 // or dispatching to the old session. Late-result fences alone are insufficient.
 for(const replace of [true,false]){
  sch=scheduler();component=load(path.join(proposed,'src/renderer/ExtensionRuntimeView.tsx'),sch.h).default;
  let seed=view('old',null,'old-generation','old-session');
  const ownedRender=()=>sch.render(component,{initial:seed,onBack(){}});
  ownedRender();const old=find(ownedRender(),'Surface').props;
  if(replace){seed=view('new',null,'new-generation','new-session');ownedRender();ownedRender()}else sch.dispose();
  const dispatched=calls.length;
  await old.onSearchTextChanged('stale input');await old.onLoadMore();await old.onInvokeAction('copy');
  assert.equal(calls.length,dispatched,'Old callbacks must not send any RPC');
  if(replace){const live=find(ownedRender(),'Surface').props;assert.equal(live.root.props.searchText,'new');assert.equal(live.actions.length,1);sch.dispose()}
 }
 console.log('Complete proposed RuntimeView, complete proposed ListRuntime, pinned Surface: ownership/rejection/generation/row-menu tests passed');
 fs.writeFileSync(path.join(__dirname,'../.work/renderer-result.json'),JSON.stringify({fullModules:6,scenarios:28})+'\n');
})().catch(e=>{console.error(e);process.exitCode=1});
