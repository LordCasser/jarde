public class BitwiseProbe {
 public static int andInt(int a,int b){return a & b;}
 public static int orInt(int a,int b){return a | b;}
 public static int xorInt(int a,int b){return a ^ b;}
 public static long andLong(long a,long b){return a & b;}
 public static long orLong(long a,long b){return a | b;}
 public static long xorLong(long a,long b){return a ^ b;}
 public static int nested(int a,int b){return (a | b) ^ (a & (b + 1));}
 public static int complement(int a){return ~a;}
 public static long complementLong(long a){return ~a;}
 public static boolean andBoolean(boolean a,boolean b){return a & b;}
 public static boolean orBoolean(boolean a,boolean b){return a | b;}
 public static boolean xorBoolean(boolean a,boolean b){return a ^ b;}
 public static boolean constant(boolean a){return a ^ true;}
 public static boolean ordered(boolean a,boolean b){return BitwiseEffects.left(a) & BitwiseEffects.right(b);}
 public static int branch(boolean a,boolean b){if (a | b) return 7;return 9;}
 public static boolean nested(boolean a,boolean b,boolean c){return a & (b ^ (c | true));}
 public static boolean copied(boolean a,boolean b,boolean c){boolean first=a|b;boolean copy=first;boolean last=copy&c;return last;}
 public static boolean hoisted(boolean a,boolean b,boolean c){boolean x=a&b;if(c)x=a|b;return x;}
 public static int passed(boolean a,boolean b,boolean c){return BitwiseEffects.accept((a ^ b) & c);}
 public static boolean array(boolean[] a){return a[0]^a[1];}
 public static int promoted(byte a,char b){return a|b;}
 public static int integerLiteralControl(){int one=1;int zero=0;return (one & zero) ^ one;}
}
