public class RenamedChainEffects {
 int a,b,c; static int calls; static int mode; static String trace="";
 static RenamedChainEffects one=new RenamedChainEffects(),two=new RenamedChainEffects();
 static int f() { calls++; trace=trace+"F"; if(mode==1) throw new IllegalStateException(); return -37; }
 static RenamedChainEffects h() { calls++; trace=trace+"H"+calls; if(mode==2 && calls==2) throw new IllegalArgumentException(); if(mode==3 && calls==1) return null; return calls==1?one:two; }
 void retained() { trace=trace+"P"; this.a=this.b=this.c=f(); }
 void receivers() { h().a=h().b=41; }
 void expr() { this.a=(this.b=f())+1; }
 void cross() { one.a=two.b=f(); }
 static void reset(int m) { calls=0;mode=m;trace="";one=new RenamedChainEffects();two=new RenamedChainEffects(); }
}
