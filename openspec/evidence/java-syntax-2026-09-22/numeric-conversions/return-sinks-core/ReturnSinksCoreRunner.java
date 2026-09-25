public class ReturnSinksCoreRunner {
 public static void main(String[]args){
  for(int x:new int[]{-129,-128,127,128,32768,65535}){
   ReturnSinksCore p=new ReturnSinksCore();p.value=x;
   System.out.println("post:"+x+":"+p.post()+":"+p.value);
   p.value=x;System.out.println("pre:"+x+":"+p.pre()+":"+p.value);
   System.out.println("guard:"+x+":"+ReturnSinksCore.guarded(new Object(),x));
  }
  try {ReturnSinksCore.guarded(null,128);System.out.println("null:returned");}
  catch(Throwable t){System.out.println("null:"+t.getClass().getName());}
 }
}
