public class NarrowRunner {public static void main(String[]args){for(int x:new int[]{-128,-1,0,1,127}){
 System.out.println("b:"+x+":"+NarrowLocals.b((byte)x));
 System.out.println("c:"+x+":"+(int)NarrowLocals.c((char)x));
 System.out.println("s:"+x+":"+NarrowLocals.s((short)x));
 System.out.println("i:"+x+":"+NarrowLocals.integer((byte)x));
}}}
