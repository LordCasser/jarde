public class LC {
    interface Op { int run(); }
    static int plain(int n){
        class L implements Op { public int run(){ return n; } }        // 局部类捕获参数
        return new L().run();
    }
    static int two(int a, int b){
        class L implements Op { public int run(){ return a*10+b; } }   // 捕获两个参数
        return new L().run();
    }
    static int local(int n){
        int d = n*2;                                                   // 捕获真局部
        class L implements Op { public int run(){ return d+1; } }
        return new L().run();
    }
    static int named(int n){
        class Named { int f(){ return n+1; } }                          // 不实现接口的局部类
        return new Named().f();
    }
    public static void main(String[] a){ System.out.println(plain(3)+"/"+two(1,2)+"/"+local(4)+"/"+named(5)); }
}
