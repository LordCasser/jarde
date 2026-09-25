public class DeferredStructured {
 public static int branch(boolean choose){if(choose)return StructuredSupport.value();return 3;}
 public static int prefix(){if(StructuredSupport.flag())return 7;return 9;}
 public static void keepPool(){StructuredSupport.mark();}
}
