package probe;
public class DispatchAudit extends ParentAudit implements FaceAudit {
 public int read(){return super.read()+1;}
 public int faceRead(){return FaceAudit.super.read();}
 public void add(int x){super.add(x);ParentAudit.bump(100);}
 public void faceTouch(int x){FaceAudit.super.touch(x);}
 public void evaluate(){super.add(ParentAudit.argument());}
 private int secret(int x){return x+10;}
 public int other(DispatchAudit x,int v){return x.secret(v);}
}
