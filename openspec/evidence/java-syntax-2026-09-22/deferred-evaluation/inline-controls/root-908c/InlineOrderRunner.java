public class InlineOrderRunner {
 interface Task{Object run();}
 static void test(String name,int m,Task t){
  InlineOrderEffects.mode=0;InlineOrderEffects.shared=new InlineHolder(7);
  InlineOrderEffects.trace=0;InlineOrderEffects.mode=m;
  try{Object r=t.run();String v=r instanceof InlineHolder?"holder:"+((InlineHolder)r).value:String.valueOf(r);
   System.out.println(name+":"+m+":"+v+":"+InlineOrderEffects.trace+":"+InlineOrderEffects.shared.value);}
  catch(Throwable e){System.out.println(name+":"+m+":"+e.getClass().getName()+":"+(e==InlineOrderEffects.FAIL)+":"+InlineOrderEffects.trace+":"+InlineOrderEffects.shared.value);}
 }
 public static void main(String[]args){for(int m=0;m<=5;m++){
  test("void",m,()->{InlineOrderControls.voidArgument();return "done";});
  test("call",m,()->InlineOrderControls.callArgument());
  test("discard",m,()->{InlineOrderControls.discarded();return "done";});
  test("concat",m,()->InlineOrderControls.concatenate());
  test("new",m,()->InlineOrderControls.construct());
  test("receiver",m,()->InlineOrderControls.receiverAndArgument());
  test("field",m,()->InlineOrderControls.fieldAndCall());
  test("write",m,()->{InlineOrderControls.fieldWrite();return "done";});
  test("local",m,()->InlineOrderControls.local());
  test("test",m,()->InlineOrderControls.test());
 }}
}
