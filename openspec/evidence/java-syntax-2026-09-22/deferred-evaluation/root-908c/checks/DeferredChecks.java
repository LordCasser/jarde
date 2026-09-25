public class DeferredChecks {
 public static int divI(int a,int b){return a/b;}
 public static int remI(int a,int b){return a%b;}
 public static long divL(long a,long b){return a/b;}
 public static long remL(long a,long b){return a%b;}
 public static int field(CheckSupport a){return a.value;}
 public static int length(int[] a){return a.length;}
 public static void keepPool(){CheckSupport.mark();}
}
