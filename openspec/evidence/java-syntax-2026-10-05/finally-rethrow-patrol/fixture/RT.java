public class RT {
    static int rethrow(String k) throws Exception {           // finally 重抛（编译器生成 throw）
        try { if(k == null) throw new Exception("n"); return 1; }
        catch(Exception e) { System.out.println("c"); }
        finally { System.out.println("f"); }
        return 2;
    }
    static int nestedRethrow(int x) {                          // catch 内重抛同异常
        try { if(x < 0) throw new IllegalArgumentException("neg"); return x; }
        catch(IllegalArgumentException e) { throw e; }         // 重抛
    }
    static class MyEx extends RuntimeException { MyEx(String m){ super(m); } }
    static int precise(int x) {                                 // 精确 rethrow（多 catch 后 throw e）
        try { if(x == 1) throw new MyEx("1"); if(x == 2) throw new IllegalStateException("2"); return 0; }
        catch(MyEx | IllegalStateException e) { throw e; }      // multi-catch 重抛
    }
    public static void main(String[] a){ try { System.out.println(""+rethrow(null)); } catch(Exception e){ System.out.println("caught:"+e.getMessage()); } try { nestedRethrow(-1); } catch(IllegalArgumentException e){ System.out.println("nr:"+e.getMessage()); } try { precise(1); } catch(MyEx e){ System.out.println("p1:"+e.getMessage()); } try { precise(2); } catch(IllegalStateException e){ System.out.println("p2:"+e.getMessage()); } }
}
