public class BitwiseAudit {
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
}
