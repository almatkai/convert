// QA only: execute exact own TSX. Replace React scheduling and OS API only.
const fs = require('node:fs');
const path = require('node:path');
const Module = require('node:module');
const root = path.resolve(__dirname, '../.work/oracle');
const load = Module._load;
let state=[],cursor=0,preference;
const jsx=(type,props)=>({type,props:props||{}});
global.React={createElement:(type,props,...children)=>jsx(type,{...props,children:children.length===1?children[0]:children}),Fragment:'Fragment'};
const List=Object.assign(()=>{}, {Section:'List.Section',Item:'List.Item'});
Module._load=function(id,...args){
 if(id==='react')return {useState:(initial)=>{const i=cursor++;if(!(i in state))state[i]=initial;return [state[i],v=>{state[i]=v}];}};
 if(id==='react/jsx-runtime')return {jsx,jsxs:jsx,Fragment:'Fragment'};
 if(id==='@raycast/api')return {Action:{CopyToClipboard:'Copy'},ActionPanel:'ActionPanel',List,Icon:{CircleFilled:'CircleFilled'},Color:{},getPreferenceValues:()=>preference};
 return load.call(this,id,...args);
};
require(path.join(root,'node_modules/tsx/dist/cjs/index.cjs'));
let key,Command;
function run(q,p){
 const k=JSON.stringify(p);
 if(k!==key){key=k;preference=p;state=[];for(const f of Object.keys(require.cache))if(f.startsWith(path.join(root,'src')))delete require.cache[f];Command=require(path.join(root,'src/convert.tsx')).default;}
 cursor=0;let tree=Command();tree.props.onSearchTextChange(q);cursor=0;tree=Command();
 const rows=[];
 function visit(e){if(!e||typeof e!=='object')return;if(e.type==='List.Item'){
  const p=e.props;const action=p.actions.props.children; if(action.type!=='Copy')throw Error('Unexpected source action');
  // Execute the content handoff rather than deriving expected bytes from the title.
  const clipboard=[];(()=>clipboard.push(action.props.content))();
  rows.push({title:p.title,subtitle:p.subtitle||'',accessory:p.accessories[0].text,copy:clipboard[0],swatch:p.icon?.tintColor?.light??null});
 }const c=e.props?.children;for(const x of Array.isArray(c)?c:[c])visit(x);}
 visit(tree);return rows;
}
const cases=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
const out=cases.map(c=>({...c,rows:run(c.query,c.preferences||{})}));
fs.writeFileSync(process.argv[3],JSON.stringify(out)+'\n');
console.log(`Executed own TSX oracle: ${out.length} cases, ${out.reduce((n,c)=>n+c.rows.length,0)} CopyToClipboard content handoffs`);
