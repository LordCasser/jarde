public class BI {
    boolean ok = false;
    void andEq(boolean v){ ok &= v; }                    // 实例布尔 &= （DV 探针只有 int）
    void orEq(boolean v){ ok |= v; }
    boolean andUse(boolean v){ ok &= v; return ok; }     // void→值消费双位
    static boolean statAnd(boolean a, boolean b){ boolean r = a; r &= b; return r; }   // 静态局部布尔（91 前沿已测恢复——worktree 二进制备验）
    boolean earlyRet(int[] xs){ for(int x : xs){ ok &= x > 0; if(!ok){ return false; } } return true; }   // 循环内布尔复合+早退（验证循环乘积）
    public static void main(String[] a){ BI b1 = new BI(); b1.andEq(true); BI b2 = new BI(); b2.andUse(false); BI b3 = new BI(); b3.ok = true; System.out.println(""+b1.ok+"/"+b2.ok+"/"+statAnd(true,false)+"/"+b3.earlyRet(new int[]{1,-2,3})); }
}
