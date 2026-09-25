public class LambdaCaptureAdaptationSupport {
 public static int pickWithPrefix(int prefix,Object value){return -1;}
 public static int pickWithPrefix(int prefix,String value){return prefix+(value==null?0:value.length());}
 public int instancePick(String value){return value==null?0:value.length();}
}
