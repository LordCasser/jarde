public class DeferredValueOrder {
 public static int call(){return DeferredEffects.value();}
 public static int field(){return DeferredEffects.field;}
 public static int array(int[] values){return values[0];}
 public static int instanceField(OrderValue holder){return holder.value;}
 public static int arrayLength(int[] values){return values.length;}
 public static int[] newArray(int length){return new int[length];}
 public static String[] anewArray(int length){return new String[length];}
 public static int[][] multiArray(int length){return new int[length][1];}
 public static int divInt(int value){return value/DeferredEffects.divisor;}
 public static int remInt(int value){return value%DeferredEffects.divisor;}
 public static long divLong(long value){return value/DeferredEffects.longDivisor;}
 public static long remLong(long value){return value%DeferredEffects.longDivisor;}
 public static String cast(Object value){return (String)value;}
 public static OrderValue direct(){return new OrderValue();}
 public static int nested(){return DeferredEffects.left()+DeferredEffects.right();}
 public static int branch(int[] values,boolean choose){if(choose)return values[0]+DeferredEffects.right();return DeferredEffects.left();}
 public static int prefix(int[] values){if(values==null)return -1;return values.length;}
 public static void keepPool(){DeferredEffects.mark();}
}
