public class ShiftAudit {
 public static int left(int value,int distance){return value << distance;}
 public static int right(int value,int distance){return value >> distance;}
 public static int unsigned(int value,int distance){return value >>> distance;}
 public static long leftLong(long value,int distance){return value << distance;}
 public static long rightLong(long value,int distance){return value >> distance;}
 public static long unsignedLong(long value,int distance){return value >>> distance;}
 public static int nested(int value,int distance){return (value << distance) + (value >>> (distance + 1)) * 2;}
 public static int local(int value,int distance){int shifted=value << distance;return shifted ^ (shifted >>> 3);}
 public static int branch(int value,int distance,boolean choose){int shifted=value << distance;if(choose)return shifted;return value >> distance;}
 public static int byteChar(byte value,char other,int distance){return (value << distance) | (other >>> distance);}
 public static int ordered(boolean failLeft,boolean failRight,int value,int distance){return ShiftEffects.left(failLeft,value) << ShiftEffects.right(failRight,distance);}
}
