public class PartialArrays {
 public static int[][] held;
 public static int[][] primitive(int n){return new int[n][];}
 public static String[][] reference(int n){return new String[n][];}
 public static int[][][] prefix(int a,int b){return new int[a][b][];}
 public static Object[][][] referencePrefix(int a,int b){return new Object[a][b][];}
 public static int[][] local(int n){int[][] value=new int[n][];return value;}
 public static String overload(int n){return PartialArrayEffects.pick(new int[n][]);}
 public static void field(int n){held=new int[n][];}
 public static int length(int n){return new int[n][].length;}
 public static int[][][] effects(int a,int b){return new int[PartialArrayEffects.dim(1,a)][PartialArrayEffects.dim(2,b)][];}
 public static int[][] complete(int a,int b){return new int[a][b];}
}
