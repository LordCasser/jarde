public class CE {
    static int sc = init("sc", 1);
    int ic = init("ic", 2);
    static int init(String n, int v){ System.out.println(n); return v; }
    CE(int x){ if(x < 0) throw new IllegalArgumentException("neg:"+x); System.out.println("ctor ok:"+x); }
    CE(){ this(-1); System.out.println("never"); }   // 委派目标抛异常
    public static void main(String[] a){ try { new CE(5); new CE(); } catch(IllegalArgumentException e){ System.out.println("caught:"+e.getMessage()); } }
}
