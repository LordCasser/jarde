public class BitwiseRunner {
 public static void main(String[] args){
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})for(boolean c:new boolean[]{false,true}){
   System.out.println("nested:"+a+":"+b+":"+c+"="+BitwiseAudit.nested(a,b,c));
   System.out.println("copied:"+a+":"+b+":"+c+"="+BitwiseAudit.copied(a,b,c));
   System.out.println("hoisted:"+a+":"+b+":"+c+"="+BitwiseAudit.hoisted(a,b,c));
   System.out.println("passed:"+a+":"+b+":"+c+"="+BitwiseAudit.passed(a,b,c));
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})System.out.println("array:"+a+":"+b+"="+BitwiseAudit.array(new boolean[]{a,b}));
  for(byte a:new byte[]{-128,-1,0,1,127})for(char b:new char[]{0,1,32768,65535})System.out.println("promoted:"+a+":"+(int)b+"="+BitwiseAudit.promoted(a,b));
 }
}
