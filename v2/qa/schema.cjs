const fs=require('node:fs'),path=require('node:path'),assert=require('node:assert/strict');
const Ajv=require('../.work/schema-tools/node_modules/ajv/dist/2020').default;
const formats=require('../.work/schema-tools/node_modules/ajv-formats');
const pinned=JSON.parse(fs.readFileSync(path.join(__dirname,'fixtures/host/schemas/extension-v3.schema.json')));
function compile(schema){const a=new Ajv({allErrors:true,strict:false});formats(a);return a.compile(schema);}
const validate=compile(pinned),manifest=JSON.parse(fs.readFileSync(path.join(__dirname,'../extension.json')));
assert(validate(manifest),JSON.stringify(validate.errors));let negative=0,positive=1;
function bad(m){assert(!validate(m),JSON.stringify(m));negative++;}
for(const key of pinned.required){const m=structuredClone(manifest);delete m[key];bad(m);}
for(const [key,value] of Object.entries({schemaVersion:2,id:'com.tezbar.bad.convert',slug:'../convert',runtime:'javascript',execution:'library',protocolVersion:2,repository:'not a URI',capabilities:['arbitrary'],replaces:['other'],platforms:['Plan9'],extra:true})){bad({...structuredClone(manifest),[key]:value});}
for(const patch of [{name:''},{platforms:[]},{runtime:'wat'},{concurrency:'busy'},{timeoutMs:0},{timeoutMs:300001},{refreshMs:499},{launcher:{recommendation:2001}}]){const m=structuredClone(manifest);Object.assign(m.commands[0],patch);bad(m);}
for(const artifacts of [{'bad-target':{}},{'aarch64-apple-darwin':{url:'http://example.com',sha256:'x',executable:'../escape',executableSha256:'x'}}])bad({...manifest,artifacts});
for(const file of ['../escape','/absolute','a\\b','a//b','a/./b','a:b'])bad({...manifest,files:{[file]:'0'.repeat(64)}});
for(const platform of ['macOS','Windows']){const m=structuredClone(manifest);m.verification=[{platform,state:'not-run'},{platform:platform==='macOS'?'Windows':'macOS',state:'not-run'}];assert(validate(m));positive++;}
const packaged=path.join(__dirname,'../.work/dev-extracted/extension.json');if(fs.existsSync(packaged)){assert(validate(JSON.parse(fs.readFileSync(packaged))),JSON.stringify(validate.errors));positive++;}
const out={positive,negative,pinnedSchema:true};fs.writeFileSync(path.join(__dirname,'schema-result.json'),JSON.stringify(out,null,2)+'\n');console.log(`Complete schema: ${positive} positive, ${negative} negative cases; exact pinned schema`);
