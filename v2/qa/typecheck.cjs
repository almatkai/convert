const fs=require('node:fs'),path=require('node:path'),ts=require('../.work/oracle/node_modules/typescript');
const root=path.resolve(__dirname,'..'),work=path.join(root,'.work/typecheck');fs.mkdirSync(work,{recursive:true});
const options={target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ESNext,moduleResolution:ts.ModuleResolutionKind.Bundler,jsx:ts.JsxEmit.ReactJSX,strict:true,skipLibCheck:true,noEmit:true,noUncheckedIndexedAccess:true,noUnusedLocals:true,noUnusedParameters:true,types:[],typeRoots:[path.join(root,'.work/oracle/node_modules/@types')],baseUrl:path.join(root,'.work/oracle/node_modules'),paths:{react:['@types/react/index.d.ts'],'react/jsx-runtime':['@types/react/jsx-runtime.d.ts']}};
const result={};
for(const variant of ['baseline','proposed']){
 const dest=path.join(work,variant);fs.mkdirSync(dest,{recursive:true});fs.cpSync(path.join(__dirname,'fixtures/host/src'),path.join(dest,'src'),{recursive:true});
 if(variant==='proposed')fs.cpSync(path.join(root,'integration/proposed/src'),path.join(dest,'src'),{recursive:true});
 // Ambient JSX alias only: pinned host uses @types/react 18; oracle lock has 19.
 fs.writeFileSync(path.join(dest,'context.d.ts'),`import type { RaymesApi } from './src/shared/desktop-api'; import type React from 'react'; declare global { namespace JSX { type Element=React.ReactElement; } interface Window { tezbar: RaymesApi } }`);
 const files=['context.d.ts','src/renderer/ExtensionRuntimeView.tsx','src/renderer/src/raycast-api/list-runtime.tsx','src/renderer/src/raycast-api/index.tsx','src/shared/extensionRuntime.ts','src/shared/desktop-api.ts'].map(f=>path.join(dest,f));
 const program=ts.createProgram(files,options);result[variant]=ts.getPreEmitDiagnostics(program).map(d=>({file:d.file?.fileName.replace(dest+'/', ''),line:d.file?d.file.getLineAndCharacterOfPosition(d.start).line+1:0,code:d.code,message:ts.flattenDiagnosticMessageText(d.messageText,' ')}));
}
fs.writeFileSync(path.join(root,'qa/typecheck-diagnostics.json'),JSON.stringify(result,null,2)+'\n');
console.log(`Isolated full-module compiler context: baseline ${result.baseline.length}, proposal ${result.proposed.length} diagnostics (see retained report; not a host build).`);
for(const d of result.proposed.filter(d=>d.file?.includes('runtimeInputOwnership')||d.file==='src/renderer/ExtensionRuntimeView.tsx'))console.log(d);
// Closed typed wire probes bind both actual interface definitions and additive revision.
const source=fs.readFileSync(path.join(root,'integration/proposed/src/shared/desktop-api.ts'),'utf8');if(!source.includes('clientInputRevision?: number'))throw Error('missing RaymesApi wire revision');
const wire=fs.readFileSync(path.join(root,'integration/proposed/src/shared/extensionRuntime.ts'),'utf8');if(!wire.includes('clientInputRevision?: number'))throw Error('missing runtime wire revision');

const known=new Set(result.baseline.map(d=>d.file+'|'+d.code+'|'+d.message)); const novel=result.proposed.filter(d=>!known.has(d.file+'|'+d.code+'|'+d.message)); if(novel.length)throw Error('New compiler diagnostics: '+JSON.stringify(novel));
