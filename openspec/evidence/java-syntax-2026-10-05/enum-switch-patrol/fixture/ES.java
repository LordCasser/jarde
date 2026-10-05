public class ES {
    enum Color { RED, GREEN, BLUE }
    static String plain(Color c){ switch(c){ case RED: return "r"; case GREEN: return "g"; case BLUE: return "b"; } return "?"; } // 完整覆盖+default 省略
    static int withDef(Color c){ int v; switch(c){ case RED: v=1; break; case GREEN: v=2; break; default: v=0; } return v; } // 共享尾变量+default
    static String nested(Color c, int n){ switch(c){ case RED: switch(n){ case 1: return "r1"; default: return "rN"; } case GREEN: return "g"; default: return "x"; } } // 嵌套 switch
    public static void main(String[] a){ System.out.println(""+plain(Color.RED)+"/"+withDef(Color.BLUE)+"/"+nested(Color.RED,1)+"/"+nested(Color.GREEN,9)); }
}
