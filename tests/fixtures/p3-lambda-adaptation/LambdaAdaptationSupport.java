public class LambdaAdaptationSupport {
 public static int calls;
 public static Object current;
 public static int pick(Object value){return 10;}
 public static int pick(String value){return 11;}
 public static int wider(Object value){calls++;return 12;}
 public static int pickArray(Object[] value){return 13;}
 public static int pickArray(String[] value){return 14;}
 public int instancePick(Object value){calls++;return 20;}
 public int instancePick(String value){calls++;return 21;}
 @SuppressWarnings("unchecked") public static <T> T genericValue(){return (T)current;}
}
