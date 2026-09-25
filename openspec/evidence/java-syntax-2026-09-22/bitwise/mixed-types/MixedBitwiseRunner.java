public class MixedBitwiseRunner {
 public static void main(String[] args){
  for(boolean a:new boolean[]{false,true})for(int b:new int[]{0,1,2,-1})System.out.println(a+":"+b+"="+MixedBitwise.value(a,b));
 }
}