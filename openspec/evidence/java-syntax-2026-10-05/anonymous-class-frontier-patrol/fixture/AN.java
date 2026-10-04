public class AN {
    static class Base { void hi(){ System.out.println("base"); } }
    interface Greeter { void greet(); }
    void plain() { System.out.println("plain"); }
    Base mkBase() { return new Base(){ void hi(){ System.out.println("anon-base"); } }; }   // 匿名子类
    Greeter mkGreet() { return new Greeter(){ public void greet(){ System.out.println("anon-iface"); } }; } // 匿名接口
    Greeter cap(final int n) { return new Greeter(){ public void greet(){ System.out.println("cap:"+n); } }; } // 带捕获
    public static void main(String[] a){
        AN x = new AN();
        x.plain(); x.mkBase().hi(); x.mkGreet().greet(); x.cap(7).greet();
    }
}
