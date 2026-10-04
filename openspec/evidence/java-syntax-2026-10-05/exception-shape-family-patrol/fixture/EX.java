public class EX {
    static String multi(String k){ try { return k.trim(); } catch(IllegalStateException | NullPointerException e){ return "multi:"+e.getClass().getSimpleName(); } } // multi-catch
    static int finReturn(int n){ try { return n; } finally { if(n==0) return -1; } }     // finally 内 return（吞 try 值）
    static String finThrow(int n){ try { if(n<0) throw new RuntimeException("neg"); return "ok"; } finally { System.out.println("ft"); } } // finally 副作用+正常抛出
    static int nestedFin(int n){ try { return 10/n; } finally { try { System.out.println("in"); } finally { System.out.println("out"); } } } // finally 嵌 finally
    public static void main(String[] a){ System.out.println(""+multi(" x ")+"/"+finReturn(0)+"/"+finThrow(1)+"/"+nestedFin(2)); System.out.println(multi(null)); }
}
