public class ShiftBasicRunner {
 private static void reset(){ShiftEffects.trace=0;}
 private static void out(String name,Object value){System.out.println(name+"="+value);}
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-12345,-1,0,1,12345,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  int[] distances={-65,-33,-1,0,1,31,32,63,64,65,127};
  for(int i=0;i<ints.length;i++)for(int d:distances){
   out("iL:"+i+":"+d,ShiftCore.left(ints[i],d));
   out("iR:"+i+":"+d,ShiftCore.right(ints[i],d));
   out("iU:"+i+":"+d,ShiftCore.unsigned(ints[i],d));
   out("nested:"+i+":"+d,ShiftCore.nested(ints[i],d));
   out("local:"+i+":"+d,ShiftCore.local(ints[i],d));
   out("branchT:"+i+":"+d,ShiftCore.branch(ints[i],d,true));
   out("branchF:"+i+":"+d,ShiftCore.branch(ints[i],d,false));
  }
  for(int i=0;i<longs.length;i++)for(int d:distances){
   out("jL:"+i+":"+d,ShiftCore.leftLong(longs[i],d));
   out("jR:"+i+":"+d,ShiftCore.rightLong(longs[i],d));
   out("jU:"+i+":"+d,ShiftCore.unsignedLong(longs[i],d));
  }
  for(byte value:new byte[]{Byte.MIN_VALUE,-1,0,1,Byte.MAX_VALUE})for(char other:new char[]{0,1,65535}){
   out("byteChar:"+value+":"+(int)other,ShiftCore.byteChar(value,other,3));
  }
  for(boolean left:new boolean[]{false,true})for(boolean right:new boolean[]{false,true}){
   reset();try{out("ordered:"+left+":"+right,ShiftCore.ordered(left,right,3,1)+":"+ShiftEffects.trace);}
   catch(RuntimeException error){out("ordered:"+left+":"+right,error.getClass().getName()+":"+error.getMessage()+":"+ShiftEffects.trace);}
  }
 }
}
