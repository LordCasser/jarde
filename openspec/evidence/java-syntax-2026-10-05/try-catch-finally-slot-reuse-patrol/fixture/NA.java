public class NA {
    static int a(int n){ try { return 100/n; } catch(Exception e){ try { return 1; } catch(Exception x){ return -1; } } }   // catch 内嵌 try（无 finally、无 throw）
    static int b(int n){ try { return 100/n; } catch(Exception e){ return 1; } finally { System.out.println("f"); } }        // 平铺 try/catch/finally（对照）
    static int c(int n){ try { return 100/n; } catch(Exception e){ try { return nested2(n); } catch(Exception x){ return -1; } } finally { System.out.println("f:"+n); } } // 嵌+finally
    static int nested2(int n){ return n; }
    public static void main(String[] x){ System.out.println(""+a(0)+"/"+b(0)+"/"+c(0)); }
}
