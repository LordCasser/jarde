package probe;
public class DispatchRunner {
 public static void main(String[] args){
  DispatchAudit x=new DispatchAudit();
  System.out.println("read="+x.read());System.out.println("faceRead="+x.faceRead());
  ParentAudit.count=0;x.add(2);System.out.println("voidSuper="+ParentAudit.count);
  ParentAudit.count=0;x.faceTouch(5);System.out.println("voidInterface="+ParentAudit.count);
  ParentAudit.count=0;x.evaluate();System.out.println("argument="+ParentAudit.count);
  System.out.println("privateOther="+x.other(new DispatchAudit(),7));
  try{x.other(null,1);}catch(Throwable e){System.out.println("null="+e.getClass().getName());}
 }
}
