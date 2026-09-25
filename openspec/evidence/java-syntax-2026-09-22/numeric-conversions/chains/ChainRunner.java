public class ChainRunner {
 public static void main(String[]args){
  for(int x:new int[]{Integer.MIN_VALUE,-65537,-1,0,1,16777217,Integer.MAX_VALUE}){
   System.out.println("i:"+x+":"+ConversionChain.intRound(x));System.out.println("bc:"+x+":"+ConversionChain.byteChar(x));
  }
  for(long x:new long[]{Long.MIN_VALUE,-9007199254740993L,-16777217L,-1L,0L,1L,16777217L,9007199254740993L,Long.MAX_VALUE}){
   System.out.println("lf:"+x+":"+ConversionChain.longFloatRound(x));System.out.println("ld:"+x+":"+ConversionChain.longDoubleRound(x));
  }
  for(double x:new double[]{Double.NEGATIVE_INFINITY,-Double.MAX_VALUE,-65537.75d,-1.5d,-0.0d,0.0d,Double.MIN_VALUE,1.5d,65535.75d,Double.MAX_VALUE,Double.NaN,Double.POSITIVE_INFINITY}){
   String id=Long.toHexString(Double.doubleToRawLongBits(x));System.out.println("df:"+id+":"+Long.toHexString(Double.doubleToRawLongBits(ConversionChain.doubleFloatRound(x))));System.out.println("dc:"+id+":"+(int)ConversionChain.doubleChar(x));
  }
  for(float x:new float[]{Float.NEGATIVE_INFINITY,-Float.MAX_VALUE,-0.0f,0.0f,Float.MIN_VALUE,1.5f,Float.MAX_VALUE,Float.NaN,Float.POSITIVE_INFINITY})System.out.println("fb:"+Integer.toHexString(Float.floatToRawIntBits(x))+":"+ConversionChain.floatByte(x));
  for(int x:new int[]{Integer.MIN_VALUE,Integer.MAX_VALUE})for(boolean l:new boolean[]{false,true})for(boolean r:new boolean[]{false,true}){
   ConversionEffects.trace=0;try{System.out.println("order:"+x+":"+l+":"+r+":"+ConversionChain.ordered(x,l,r)+":"+ConversionEffects.trace);}catch(Throwable e){System.out.println("order:"+x+":"+l+":"+r+":"+e.getClass().getName()+":"+ConversionEffects.trace);}
  }
 }
}
