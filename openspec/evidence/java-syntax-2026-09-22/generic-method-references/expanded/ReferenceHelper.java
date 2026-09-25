public class ReferenceHelper {
 public int pick(Object value){return 20;}
 public int pick(String value){return 21;}
 public int pick(Object[] value){return 22;}
 public int pick(String[] value){return 23;}
 public static ReferenceHelper make(){return new ReferenceHelper();}
 public static ReferenceHelper nullReceiver(){return null;}
 public static String text(){return "text";}
 @SuppressWarnings("unchecked") public static <T> T genericText(){return (T)"generic";}
}
