"""Input-only exact-rational point oracle for pure-u derivatives of a NURBS offset.

Cox-de Boor basis and formal division give B and B_v as rational Taylor coefficients.
For C=B_u x B_v, Q=C.C, expand (Q/Q0)^(-1/2) to order3. Every requested
component is A + R/sqrt(Q0), with A,R,Q0 exact Fractions. Integer isqrt brackets
the sole radical; no floating arithmetic, kernel output or convergence heuristic
enters the expected-value construction. This is a local point enclosure, not a
certificate over a face. The driver explicitly reports unsupported script states.
"""
from fractions import Fraction as F
from math import factorial,isqrt,isfinite
from functools import lru_cache
from decimal import Decimal,localcontext,ROUND_FLOOR,ROUND_CEILING
import json,pathlib,copy,collections,sys
sys.set_int_max_str_digits(0)
N=4
NOW=-float.fromhex('0x1.fffffffffffffp+1023')
class Unsupported(Exception):pass

def f(x):
 if isinstance(x,float):
  if not isfinite(x):raise Unsupported('nonfinite input')
  return F.from_float(x)
 return F(x)
def zero():return [F(0)]*(N+1)
def add(a,b):return [x+y for x,y in zip(a,b)]
def neg(a):return [-x for x in a]
def scale(a,s):return [s*x for x in a]
def mul(a,b):return [sum((a[k]*b[i-k] for k in range(i+1)),F(0)) for i in range(N+1)]
def div(a,b):
 if not b[0]:raise Unsupported('rational denominator vanishes')
 c=zero()
 for i in range(N+1):c[i]=(a[i]-sum((b[k]*c[i-k] for k in range(1,i+1)),F(0)))/b[0]
 return c
def derivative(a):return [(i+1)*a[i+1] for i in range(N)]+[F(0)]
def cross(a,b):return [add(mul(a[1],b[2]),neg(mul(a[2],b[1]))),add(mul(a[2],b[0]),neg(mul(a[0],b[2]))),add(mul(a[0],b[1]),neg(mul(a[1],b[0])))]
def basis(knots,p,t,count,left):
 if t==knots[p]:left=True
 if t==knots[count]:left=False
 x=zero();x[0]=t;x[1]=1
 @lru_cache(None)
 def one(i,d):
  if d==0:
   inside=(knots[i]<=t<knots[i+1]) if left else (knots[i]<t<=knots[i+1]);v=zero();v[0]=F(int(inside));return tuple(v)
  out=zero()
  a=knots[i+d]-knots[i]
  if a:
   c=x.copy();c[0]-=knots[i];out=add(out,scale(mul(c,one(i,d-1)),1/a))
  b=knots[i+d+1]-knots[i+1]
  if b:
   c=neg(x);c[0]+=knots[i+d+1];out=add(out,scale(mul(c,one(i+1,d-1)),1/b))
  return tuple(out)
 return [list(one(i,p)) for i in range(count)]
def flatten(off):
 d=f(off['d']);base=off['base']
 while 'offset' in base:
  inner=base['offset'];d+=f(inner['d']);base=inner['base']
 while True:
  if 'step' in base:base=base['step']['base']
  elif 'bounded' in base:base=base['bounded']['basis']
  else:break
 if 'bspline' not in base:raise Unsupported('non-B-spline or wrapped nested-offset provider')
 return base['bspline'],d

def expressions(off,step):
 b,d=flatten(off);p,q=b['p'],b['q'];net=b['net'];nu,nv=len(net),len(net[0]);U,V=list(map(f,b['u'])),list(map(f,b['v']));u,v=map(f,step['uv'])
 if not (1<=p<=3 and 1<=q<=3):raise Unsupported('degree outside1..3')
 if len(U)!=nu+p+1 or len(V)!=nv+q+1 or any(len(row)!=nv for row in net):raise Unsupported('unexpected net/knots shape')
 if not (U[p]<=u<=U[nu] and V[q]<=v<=V[nv]):raise Unsupported('query outside original domain; extension/clamp not modeled')
 rational=any(c[3]!=NOW for row in net for c in row)
 if rational and any(c[3]==NOW for row in net for c in row):raise Unsupported('mixed weight markers')
 B=basis(U,p,u,nu,step.get('ul',True));D=basis(V,q,v,nv,step.get('vl',True));H=[zero() for _ in range(4)];Hv=[zero() for _ in range(4)]
 for i in range(nu):
  for j in range(nv):
   cp=list(map(f,net[i][j]));cp[2]=F(0) if net[i][j][2]==-NOW else cp[2]
   if not rational:cp[3]=F(1)
   for a in range(4):H[a]=add(H[a],scale(B[i],cp[a]*D[j][0]));Hv[a]=add(Hv[a],scale(B[i],cp[a]*D[j][1]))
 if not rational:assert H[3]==[F(1),F(0),F(0),F(0),F(0)]
 S=[div(h,H[3]) for h in H[:3]];Sv=[div(add(hv,neg(mul(s,Hv[3]))),H[3]) for s,hv in zip(S,Hv[:3])];C=cross(list(map(derivative,S)),Sv);Q=add(add(mul(C[0],C[0]),mul(C[1],C[1])),mul(C[2],C[2]));q0=Q[0]
 if q0<=0:raise Unsupported('no regular normal at this point')
 E=scale(Q,1/q0);E[0]-=1;power=zero();power[0]=1;R=zero()
 for k,c in enumerate([F(1),F(-1,2),F(3,8),F(-5,16)]):
  if k:power=mul(power,E)
  R=add(R,scale(power,c))
 J=[mul(c,R) for c in C]
 out={n:[(s[n]*factorial(n),d*j[n]*factorial(n)) for s,j in zip(S,J)] for n in [2,3]}
 point=max(abs(a[0]) for a in S);tol=10*f(1e-12)*(1+point);bu2=sum((a[1]*a[1] for a in S),F(0));bv2=sum((a[0]*a[0] for a in Sv),F(0))
 def threshold(z):
  return 'far_above' if z>tol*tol*10**6 else 'far_below' if z*10**6<tol*tol else 'near_or_between_margins'
 return q0,out,{'provider':'rational' if rational else 'polynomial','domain_position':'interior' if U[p]<u<U[nu] and V[q]<v<V[nv] else 'boundary','u_at_knot':u in U,'v_at_knot':v in V,'exact_point_maxabs':str(point),'threshold_estimate':str(tol),'bu_squared':str(bu2),'bv_squared':str(bv2),'normal_vs_threshold':threshold(q0),'bu_vs_threshold':threshold(bu2),'bv_vs_threshold':threshold(bv2),'threshold_note':'Exact-definition margins relative to the source threshold; not an observed runtime branch trace.'}

def bracket(q,bits):
 a,b=q.numerator,q.denominator;aa,bb=isqrt(a),isqrt(b)
 if aa*aa==a and bb*bb==b:return F(aa,bb),F(aa,bb),None
 k=bits-(a.bit_length()-b.bit_length())//2
 m=isqrt((a<<(2*k))//b if k>=0 else a//(b<<(-2*k)))
 lo=F(m,1<<k) if k>=0 else F(m*(1<<(-k)));hi=F(m+1,1<<k) if k>=0 else F((m+1)*(1<<(-k)))
 assert 0<lo and lo*lo<=q<=hi*hi
 return lo,hi,{'bits':bits,'integer_root_floor':str(m),'binary_shift':k}
def enclosure(a,b,q):
 if not b:return a,a,None
 if a*b<0 and a*a*q==b*b:return F(0),F(0),{'exact_zero_identity':True}
 for bits in [256,512,1024,2048,4096]:
  l,h,c=bracket(q,bits);x,y=sorted([a+b/h,a+b/l])
  if x==y or (x*y>0 and y-x<=max(abs(x),abs(y))*F(1,10**40)):return x,y,c
 raise Unsupported('radical enclosure unresolved within4096bits')
def dec(x,up):
 with localcontext() as c:c.prec=85;c.rounding=ROUND_CEILING if up else ROUND_FLOOR;return str(Decimal(x.numerator)/Decimal(x.denominator))
def compare(actual,a,b,q):
 lo,hi,cert=enclosure(a,b,q);r={'expected_interval':[dec(lo,False),dec(hi,True)],'expression':[str(a),str(b)],'root_bracket':cert}
 if isinstance(actual,str) or not isfinite(actual):r.update(status='nonfinite_actual',actual=actual);return r
 v=f(actual)
 def rel(x):return abs(v-x)/max(abs(v),abs(x)) if v or x else F(0)
 error=max(rel(lo),rel(hi));r.update(status='agree_strict_relative' if error<=F(1,10**9) else 'strict_relative_disagreement',actual=actual,relative_error_upper=float(error));return r

def selftest():
 U=list(map(F,[0,0,F(1,2),1,1]));cp=[F(0),F(1),F(3)]
 for left,want in [(True,F(4)),(False,F(2))]:
  bs=basis(U,1,F(1,2),3,left);assert sum((p*b[1] for p,b in zip(cp,bs)),F(0))==want
 d=0.3;sent=NOW
 inp={'d':d,'base':{'bspline':{'p':2,'q':1,'u':[0,0,0,1,1,1],'v':[0,0,1,1],'net':[[[u,v,z,sent] for v in [0,1]] for u,z in [(0,0),(0.5,0),(1,1)]]}}}
 q,e,_=expressions(inp,{'uv':[0.5,0.5]});assert q==2;assert e[2][0]==(0,3*f(d));assert e[2][2]==(2,f(d));assert e[3][0]==(0,-9*f(d));assert e[3][2]==(0,3*f(d));assert compare(2.2121320343559643,*e[2][2],q)['status']=='agree_strict_relative'
 inp={'d':d,'base':{'bspline':{'p':1,'q':1,'u':[0,0,1,1],'v':[0,0,1,1],'net':[[[i*w,j*w,0,w] for j in [0,1]] for i,w in [(0,1),(1,2)]]}}}
 q,e,_=expressions(inp,{'uv':[0.5,0.5]});assert e[2][0][0]==F(-32,27);assert e[3][0][0]==F(64,27)
 for n in [2,3]:assert enclosure(*e[n][2],q)[:2]==(0,0)
 return 'Exact parabolic symbolic jets and rational-plane second/third derivatives checked; radical bracket squares checked exactly.'

def main():
 import argparse
 parser=argparse.ArgumentParser();parser.add_argument('--output',required=True);args=parser.parse_args()
 control_check=selftest();root=pathlib.Path(__file__).parent;records=[json.loads(p.read_text()) for p in sorted((root/'attribution').glob('*.json'))];data={x['case']:x for x in records};at=[x['prefix_attribution'] for x in records];out=[];counts=collections.Counter();mutating={'reverse','swap','transform','trim'}
 for row in at:
  inp=copy.deepcopy(data[row['case']]['inputs']);off=inp['off'];details={x['step']:x for x in row['steps']};unsupported_state=None
  for n,step in enumerate(inp['steps']):
   op=step['op']
   if n in details:
    rec={'case':row['case'],'step':n,'query':step}
    try:
     if unsupported_state:raise Unsupported(unsupported_state)
     q,expr,scope=expressions(off,step);rec.update(scope=scope,normal_squared=str(q));components=[]
     for change in details[n].get('changed_named_derivatives',[]):
      order,vorder=change['derivative'];assert vorder==0
      for axis,(old,new) in enumerate(zip(change['old'],change['new'])):
       a,b=expr[order][axis];z=compare(new,a,b,q);z.update(derivative=[order,0],axis=axis,old=old);components.append(z);counts[z['status']]+=1
     rec.update(status='covered',components=components)
    except Unsupported as e:rec.update(status='outside_oracle_scope',reason=str(e));counts['outside_oracle_scope']+=1
    out.append(rec)
   if op in mutating:unsupported_state='prior mutating '+op+' is not modeled'
   if op=='set_dist':off['d']=step['d']
   if op=='set_allow':off['allow']=step['b']
 pathlib.Path(args.output).write_text(json.dumps({'method':__doc__,'selftest':control_check,'counts':dict(counts),'queries':out},indent=2)+'\n');print(json.dumps(dict(counts)))
if __name__=='__main__':main()
