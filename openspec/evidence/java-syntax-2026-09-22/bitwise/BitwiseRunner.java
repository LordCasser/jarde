public class BitwiseRunner {
 public static void main(String[] args){
  int[] ints={Integer.MIN_VALUE,-1,0,1,Integer.MAX_VALUE};
  long[] longs={Long.MIN_VALUE,-1L,0L,1L,Long.MAX_VALUE};
  for(int i=0;i<ints.length;i++) {
   System.out.println("notI:"+i+"="+BitwiseAudit.complement(ints[i]));
   System.out.println("notJ:"+i+"="+BitwiseAudit.complementLong(longs[i]));
   for(int j=0;j<ints.length;j++) {
    System.out.println("andI:"+i+":"+j+"="+BitwiseAudit.andInt(ints[i],ints[j]));
    System.out.println("orI:"+i+":"+j+"="+BitwiseAudit.orInt(ints[i],ints[j]));
    System.out.println("xorI:"+i+":"+j+"="+BitwiseAudit.xorInt(ints[i],ints[j]));
    System.out.println("andJ:"+i+":"+j+"="+BitwiseAudit.andLong(longs[i],longs[j]));
    System.out.println("orJ:"+i+":"+j+"="+BitwiseAudit.orLong(longs[i],longs[j]));
    System.out.println("xorJ:"+i+":"+j+"="+BitwiseAudit.xorLong(longs[i],longs[j]));
    System.out.println("nested:"+i+":"+j+"="+BitwiseAudit.nested(ints[i],ints[j]));
   }
  }
  for(boolean a:new boolean[]{false,true})for(boolean b:new boolean[]{false,true}){
   System.out.println("andZ:"+a+":"+b+"="+BitwiseAudit.andBoolean(a,b));
   System.out.println("orZ:"+a+":"+b+"="+BitwiseAudit.orBoolean(a,b));
   System.out.println("xorZ:"+a+":"+b+"="+BitwiseAudit.xorBoolean(a,b));
   System.out.println("constant:"+a+":"+b+"="+BitwiseAudit.constant(a));
   System.out.println("branch:"+a+":"+b+"="+BitwiseAudit.branch(a,b));
   BitwiseEffects.trace=0;BitwiseEffects.throwing=0;
   System.out.println("ordered:"+a+":"+b+"="+BitwiseAudit.ordered(a,b)+":"+BitwiseEffects.trace);
  }
  for(int t=1;t<=2;t++){
   BitwiseEffects.trace=0;BitwiseEffects.throwing=t;
   try{BitwiseAudit.ordered(false,true);System.out.println("throw:"+t+"=returned");}
   catch(RuntimeException e){System.out.println("throw:"+t+"="+e.getClass().getName()+":"+e.getMessage()+":"+BitwiseEffects.trace);}
  }
 }
}
