public class XB {
    interface Throwing { int run() throws Exception; }
    static int useThrow() throws Exception { Throwing t = () -> 5 > 1 ? 1 : 2; return t.run(); }  // lambda 体经声明抛的 SAM
    static int handle(){ try { return useThrow(); } catch(Exception e){ return -1; } }
    static int interCast(Object o){ Comparable<java.io.Serializable> c = (java.io.Serializable & Comparable<java.io.Serializable>) o; return c == null ? 0 : 1; }
    public static void main(String[] a){ System.out.println(""+handle()+"/"+interCast(null)); }
}
