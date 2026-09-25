public class ConversionChain {
 public static int intRound(int x){return (int)(float)x;}
 public static long longFloatRound(long x){return (long)(float)x;}
 public static long longDoubleRound(long x){return (long)(double)x;}
 public static double doubleFloatRound(double x){return (double)(float)x;}
 public static byte floatByte(float x){return (byte)x;}
 public static char doubleChar(double x){return (char)x;}
 public static int byteChar(int x){return (char)(byte)x;}
 public static long ordered(int x,boolean left,boolean right){return (long)ConversionEffects.left(x,left)+(long)ConversionEffects.right(x,right);}
}
