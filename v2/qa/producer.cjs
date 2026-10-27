// Execute complete immutable V1 action functions, replacing only OS endpoints.
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm'),assert=require('node:assert/strict');
const ts=require('../.work/oracle/node_modules/typescript');
const file=path.join(__dirname,'fixtures/host/src/main/extension-runner.ts');
const text=fs.readFileSync(file,'utf8'),ast=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true);
const names=new Set(['normalizeActionTitle','stableActionId','parseShortcut','pushEffect','copyToSystemClipboard','registerAction']);
let source='';for(const n of ast.statements)if(ts.isFunctionDeclaration(n)&&names.has(n.name?.text))source+=n.getFullText(ast)+'\n';
assert.equal((source.match(/function /g)||[]).length,6);
const calls=[];const context={createHash:require('node:crypto').createHash,copyClipboardContent:async v=>calls.push(v),pasteClipboardContent:()=>{throw Error('Unexpected paste')},shell:{openExternal:()=>{throw Error('Unexpected open')}},console};
vm.createContext(context);vm.runInContext(ts.transpile(source,{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}),context);
(async()=>{let count=0;for(const c of JSON.parse(fs.readFileSync(path.join(__dirname,'oracle.json'))))for(const r of c.rows){
 const s={currentActions:[],actionHandlers:new Map(),effects:[],effectMode:'execute'};
 context.registerAction('Action.CopyToClipboard',{content:r.copy},s);
 assert.equal(s.currentActions[0].title,'Copy to Clipboard');assert.equal(s.currentActions[0].kind,'copy');
 await s.actionHandlers.get(s.currentActions[0].id)();assert.equal(calls.pop(),r.copy);assert.equal(s.effects[0].value,r.copy.slice(0,2000));count++;
 }console.log(`Complete pinned registerAction/copyToSystemClipboard: ${count} callbacks; only clipboard OS delivery mocked`);
 fs.writeFileSync(path.join(__dirname,'../.work/producer-result.json'),JSON.stringify({callbacks:count,completeFunctions:[...names]})+'\n');
})().catch(e=>{console.error(e);process.exitCode=1});
