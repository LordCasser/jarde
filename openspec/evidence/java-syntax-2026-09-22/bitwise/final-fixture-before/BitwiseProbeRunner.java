public class BitwiseProbeRunner {
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  for(int i=0;i<ints.length;i++) {
   System.out.println("notI:"+i+"="+BitwiseProbe.complement(ints[i]));
   System.out.println("notJ:"+i+"="+BitwiseProbe.complementLong(longs[i]));
   for(int j=0;j<ints.length;j++) {
    System.out.println("andI:"+i+":"+j+"="+BitwiseProbe.andInt(ints[i],ints[j]));
    System.out.println("orI:"+i+":"+j+"="+BitwiseProbe.orInt(ints[i],ints[j]));
    System.out.println("xorI:"+i+":"+j+"="+BitwiseProbe.xorInt(ints[i],ints[j]));
    System.out.println("andJ:"+i+":"+j+"="+BitwiseProbe.andLong(longs[i],longs[j]));
    System.out.println("orJ:"+i+":"+j+"="+BitwiseProbe.orLong(longs[i],longs[j]));
    System.out.println("xorJ:"+i+":"+j+"="+BitwiseProbe.xorLong(longs[i],longs[j]));
    System.out.println("nestedI:"+i+":"+j+"="+BitwiseProbe.nested(ints[i],ints[j]));
   }
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true}){
   System.out.println("andZ:"+a+":"+b+"="+BitwiseProbe.andBoolean(a,b));
   System.out.println("orZ:"+a+":"+b+"="+BitwiseProbe.orBoolean(a,b));
   System.out.println("xorZ:"+a+":"+b+"="+BitwiseProbe.xorBoolean(a,b));
   System.out.println("constant:"+a+":"+b+"="+BitwiseProbe.constant(a));
   System.out.println("branch:"+a+":"+b+"="+BitwiseProbe.branch(a,b));
   BitwiseEffects.trace=0;BitwiseEffects.throwing=0;
   System.out.println("ordered:"+a+":"+b+"="+BitwiseProbe.ordered(a,b)+":"+BitwiseEffects.trace);
  }
  for(int t=1;t<=2;t++){
   BitwiseEffects.trace=0;BitwiseEffects.throwing=t;
   try{BitwiseProbe.ordered(false,true);System.out.println("throw:"+t+"=returned");}
   catch(RuntimeException e){System.out.println("throw:"+t+"="+e.getClass().getName()+":"+e.getMessage()+":"+BitwiseEffects.trace);}
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})for(boolean c:new boolean[]{false,true}){
   System.out.println("nestedZ:"+a+":"+b+":"+c+"="+BitwiseProbe.nested(a,b,c));
   System.out.println("copied:"+a+":"+b+":"+c+"="+BitwiseProbe.copied(a,b,c));
   System.out.println("hoisted:"+a+":"+b+":"+c+"="+BitwiseProbe.hoisted(a,b,c));
   System.out.println("passed:"+a+":"+b+":"+c+"="+BitwiseProbe.passed(a,b,c));
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true})System.out.println("array:"+a+":"+b+"="+BitwiseProbe.array(new boolean[]{a,b}));
  for(byte a:new byte[]{-128,-1,0,1,127})for(char b:new char[]{0,1,32768,65535})System.out.println("promoted:"+a+":"+(int)b+"="+BitwiseProbe.promoted(a,b));
 }
}
