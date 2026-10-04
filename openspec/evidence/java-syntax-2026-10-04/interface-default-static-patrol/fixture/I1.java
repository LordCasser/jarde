import java.util.*;
public class I1 {
    interface Greeter {
        String greet(String who);
        default String hello() { return greet("world"); }           // default 调用抽象方法
        default int count(String s) { int n=0; for(char c: s.toCharArray()) if(c=='a') n++; return n; }  // default 带循环
        static Greeter upper() { return who -> who.toUpperCase(); }   // static 返回 lambda
        default String compose() { return "x" + hello() + count("banana"); }  // default 拼接+调用
    }
    static class Impl implements Greeter {
        public String greet(String who) { return "hi " + who; }
    }
    public static void main(String[] a) {
        Greeter g = new Impl();
        System.out.println(g.hello());
        System.out.println(g.count("banana"));
        System.out.println(Greeter.upper().greet("bob"));
        System.out.println(g.compose());
    }
}
