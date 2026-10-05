public class CO {
    static String order(int x){
        try { if(x == 0) throw new IllegalStateException("ise"); if(x == 1) throw new IllegalArgumentException("iae"); if(x == 2) throw new RuntimeException("re"); return "ok"; }
        catch(IllegalStateException e){ return "ISE:" + e.getMessage(); }      // 子类在前
        catch(IllegalArgumentException e){ return "IAE:" + e.getMessage(); }   // 平级
        catch(RuntimeException e){ return "RE:" + e.getMessage(); }            // 父类在后（顺序可观察）
        finally { }
    }
    static int fib(int n){ return n < 2 ? n : fib(n-1) + fib(n-2); }           // 自递归
    static boolean even(int n){ return n == 0 ? true : odd(n-1); }             // 互递归
    static boolean odd(int n){ return n == 0 ? false : even(n-1); }
    public static void main(String[] a){ System.out.println(""+order(0)+"/"+order(1)+"/"+order(2)+"/"+order(9)+"/"+fib(10)+"/"+even(10)+"/"+odd(7)); }
}
