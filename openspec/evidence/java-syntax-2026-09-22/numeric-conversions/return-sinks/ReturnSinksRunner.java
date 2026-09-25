public class ReturnSinksRunner {
 public static void main(String[]args){
  for(int x:new int[]{-129,-128,127,128,32768,65535}){
   ReturnSinks p=new ReturnSinks();p.value=x;
   System.out.println("post:"+x+":"+p.post()+":"+p.value);
   p.value=x;System.out.println("pre:"+x+":"+p.pre()+":"+p.value);
   System.out.println("guard:"+x+":"+ReturnSinks.guarded(new Object(),x));
   System.out.println("left:"+x+":"+ReturnSinks.choice(true,x,17));
   System.out.println("right:"+x+":"+ReturnSinks.choice(false,17,x));
  }
  try {ReturnSinks.guarded(null,128);System.out.println("null:returned");}
  catch(Throwable t){System.out.println("null:"+t.getClass().getName());}
 }
}
