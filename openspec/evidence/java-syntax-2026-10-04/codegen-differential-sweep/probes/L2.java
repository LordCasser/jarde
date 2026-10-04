import java.util.function.*;
public class L2 {
    interface Two { int f(int a, int b); }
    static Two block(int k){ return (a,b) -> { int t = a+b; return t+k; }; }   // 多语句块体（5.2 域）
    static Consumer<String> stmt(){ return s -> { System.out.println(s); } ; } // 语句体
    public static void main(String[] a){ System.out.println(block(1).f(2,3)); stmt().accept("x"); }
}
