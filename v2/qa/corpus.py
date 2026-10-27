import random,json,pathlib
r=random.Random(6734);cases=[]
def add(q,p=None): cases.append(dict(query=q,preferences=p or {}))
edge=['',' ','\t','😀','１２px','١٢px','16px','16PX','1REM','-.5rem','-12px','1.5rem','1,5rem','16 px',' 1.5rem ','prefix12px','1e3px','1e-7px','NaNpx','Infinitypx','10px 2rem 3pt','0rem','0px','0pt','0.00000000000000000001rem','999999999999999999999999999999999999999rem','#069','#ABc','#1234','#12345','#123456','#12345678','#1234567','#123456789',' #006699','#fafafa','#FAFAFA','#000','#00000','#00000000','rgb(255, 0, 128)','RGB(100% 0% 50% / 50%)','rgba(999,999,999,999%)','rgba(1 2 3 / .5)','rgb(1 2 3 0.5)','rgb(1,2 3)','rgb(1\u00a02\uFEFF3)','rgb(-1,2,3)','rgb(1.5,2,3)','rgb(1e2 0 0)','hsl(999 999 999 / 1000)','HSLA(360,100%,50%,0)','hsl(120 50% 75% / 999%)','hsl(0 0 50)','oklch(50% 0.1 200)','oklch(. . .)','oklch(1..2 .1 200.)','oklch(1 1 99999999999999999999999999999999999999999999999999)','oklch(0 0 0)','oklch(1 0 0)','oklch(0.5 0.1 200 / .5)']
for q in edge:add(q)
for p in ['', '16','0','-0','-16',' ', '\ufeff','NaN','Infinity','-Infinity','1e-7','1e21','0x10','0b10000','0o20','0x','abc','+2.5',None,0,False,True,[],[16],{},[1,2]]:
 for q in ['1rem','-12px','16px','0rem','12pt','1,2rem','1.0000000000000002rem']:add(q,{'basePixel':p})
for _ in range(12000):
 kind=r.randrange(7)
 if kind==0:q=f'#{r.randrange(1<<24):06x}'+(f'{r.randrange(256):02x}' if r.randrange(2) else '')
 elif kind==1:q=f'rgb({r.randrange(1000)}{r.choice(["","%"])}{r.choice([",", " "])}{r.randrange(1000)} {r.randrange(1000)} / {r.choice([".5","0","1","999%","1.0001","0.00196078431372549"])})'
 elif kind==2:q=f'hsl({r.randrange(1000)} {r.randrange(1000)}% {r.randrange(1000)}% / {r.choice(["0","50%","999",".5"])})'
 elif kind==3:q=f'oklch({r.randrange(2000)/1000} {r.randrange(1000)/1000} {r.randrange(10000)/10})'
 elif kind==4:q=f'{r.choice(["-","+"," ","", "x"])}{r.randrange(100000)/1000}{r.choice(["rem","px","pt","REM","PX"])}'
 elif kind==5:q=''.join(r.choice('12.,eE+- %()#abcdef😀\t\n') for _ in range(r.randrange(1,25)))
 else:q=f'rgb({r.randrange(256)}, {r.randrange(256)}, {r.randrange(256)})'
 add(q)
for i in range(4096):add(f'#{i:03x}')
for i in range(256):add(f'rgb({i}, {i}, {i})')
for q in ['😀12px','😀1px','é1px','١1px','1rem\n','rgb(1 2 3)\n','hsl(0 999 999 / '+('9'*350)+')','rgba(1 2 3 / '+('9'*350)+')','oklch('+('9'*350)+' 1 1)','oklch(1 . .)','oklch(. 1 1)','12\u0085px','12\u2028px','12\ufeffpx','1e21rem','-0.00001rem'] :add(q)
# Every Tailwind datum, full spacing and thresholds/ties use actual own source data.
for _,h in json.loads(pathlib.Path('v2/src/tailwind.json').read_text()):add(h)
for _,rem,px in json.loads(pathlib.Path('v2/src/spacings.json').read_text()):
 for v,u in [(rem,'rem'),(px,'px')]:
  for delta in [-0.000001,0,0.000001]:add(f'{v+delta:.6f}{u}')
pathlib.Path('v2/qa/cases.json').write_text(json.dumps(cases,ensure_ascii=False,separators=(',',':'))+'\n')
print(len(cases))
