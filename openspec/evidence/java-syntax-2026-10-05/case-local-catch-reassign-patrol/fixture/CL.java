public class CL {
    static int caseLocal(int x){                      // switch case 块内局部声明（每 case 独立作用域）
        switch(x){
            case 1: { int t = x * 10; return t + 1; }
            case 2: { int t = x * 20; return t + 2; }   // 同名 t 各 case 独立
            default: return 0;
        }
    }
    static String catchReassign(String k){             // catch 参数重新赋值
        try { if(k == null){ throw new IllegalStateException("nil"); } return "ok:" + k; }
        catch(IllegalStateException e){ e = new IllegalStateException("wrapped:" + e.getMessage()); return e.getMessage(); }
    }
    static int caseFallLocal(int x){                   // 贯穿进入局部声明 case（源级非法——t 需块；这里用块防贯穿跳过声明）
        switch(x){ case 1: { int t = 5; } case 2: { return 22; } default: return 0; }
    }
    static class MyEx extends RuntimeException { MyEx(String m, int code){ super(m + "#" + code); } }
    static String customEx(int c){ try { throw new MyEx("bad", c); } catch(MyEx e){ return e.getMessage(); } }
    public static void main(String[] a){ System.out.println(""+caseLocal(1)+"/"+caseLocal(2)+"/"+catchReassign(null)+"/"+caseFallLocal(1)+"/"+caseFallLocal(2)+"/"+customEx(7)); }
}
