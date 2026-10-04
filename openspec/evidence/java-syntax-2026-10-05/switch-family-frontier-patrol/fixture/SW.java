public class SW {
    static String s(int n){ switch(n){ case 1: return "one"; case 2: return "two"; default: return "other"; } }        // 简单 return switch
    static int t(int n){ int r; switch(n){ case 1: r=10; break; case 2: r=20; break; default: r=0; } return r; }        // fallthrough+break+共享尾
    static String str(String k){ switch(k){ case "a": return "A"; case "b": case "c": return "BC"; default: return "?"; } } // String switch 合并 case
    static int fal(int n){ switch(n){ case 1: case 2: return 12; case 3: { int x = n*2; return x; } default: return 0; } } // 合并+块作用域
    static int yld(int n){ int v = switch8(n); return v; }
    static int switch8(int n){ return n; } // placeholder for javac8-compat
    public static void main(String[] a){ System.out.println(s(1)+"/"+t(2)+"/"+str("b")+"/"+fal(3)); }
}
