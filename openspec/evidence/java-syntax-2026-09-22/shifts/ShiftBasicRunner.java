public class ShiftBasicRunner {
 private static void reset(){ShiftEffects.trace=0;}
 private static void out(String name,Object value){System.out.println(name+"="+value);}
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-12345,-1,0,1,12345,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  int[] distances={-65,-33,-1,0,1,31,32,63,64,65,127};
  for(int i=0;i<ints.length;i++)for(int d:distances){
   out("iL:"+i+":"+d,ShiftAudit.left(ints[i],d));
   out("iR:"+i+":"+d,ShiftAudit.right(ints[i],d));
   out("iU:"+i+":"+d,ShiftAudit.unsigned(ints[i],d));
   out("nested:"+i+":"+d,ShiftAudit.nested(ints[i],d));
   out("local:"+i+":"+d,ShiftAudit.local(ints[i],d));
   out("branchT:"+i+":"+d,ShiftAudit.branch(ints[i],d,true));
   out("branchF:"+i+":"+d,ShiftAudit.branch(ints[i],d,false));
  }
  for(int i=0;i<longs.length;i++)for(int d:distances){
   out("jL:"+i+":"+d,ShiftAudit.leftLong(longs[i],d));
   out("jR:"+i+":"+d,ShiftAudit.rightLong(longs[i],d));
   out("jU:"+i+":"+d,ShiftAudit.unsignedLong(longs[i],d));
  }
  for(byte value:new byte[]{Byte.MIN_VALUE,-1,0,1,Byte.MAX_VALUE})for(char other:new char[]{0,1,65535}){
   out("byteChar:"+value+":"+(int)other,ShiftAudit.byteChar(value,other,3));
  }
  for(boolean left:new boolean[]{false,true})for(boolean right:new boolean[]{false,true}){
   reset();try{out("ordered:"+left+":"+right,ShiftAudit.ordered(left,right,3,1)+":"+ShiftEffects.trace);}
   catch(RuntimeException error){out("ordered:"+left+":"+right,error.getClass().getName()+":"+error.getMessage()+":"+ShiftEffects.trace);}
  }
 }
}
