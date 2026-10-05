public class CI {
    static boolean ready = false;
    static int boot(){ if(!ready){ throw new IllegalStateException("not ready"); } return 42; }   // clinit 路径抛出
    static final int VALUE = boot();                                    // clinit 内调用抛出方法
    static int safeGet(){ try { Class.forName("CI"); } catch(Throwable t){} return VALUE; }
    static boolean unbox(Boolean b){ return b ? true : false; }          // Boolean 三元拆箱
    static int unboxIf(Boolean b){ if(b){ return 1; } return 0; }        // Boolean 条件拆箱
    static int doAnd(int n){ int c = 0; do { c++; } while(c < n && c < 100); return c; }   // do-while 复合条件
    static int whileOr(java.util.List<Integer> l){ int s = 0; int i = 0; while(i < l.size() || s < 0){ if(i < l.size()){ s += l.get(i); } i++; } return s; }  // while || 复合
    public static void main(String[] a){ System.out.println(""+safeGet()+"/"+unbox(Boolean.TRUE)+"/"+unboxIf(Boolean.FALSE)+"/"+doAnd(5)+"/"+whileOr(java.util.Arrays.asList(1,2,3))); }
}
