public class CO {
    static abstract class A {                                 // 基类 ctor 调虚方法（经典陷阱）
        int a;
        A(){ a = hook(); }
        abstract int hook();
    }
    static class B extends A {
        int b = 10;                                            // 基类 ctor 期间仍是 0
        int hook(){ return b + 1; }
    }
    static class C extends A {                                 // 实例初始化块对照
        int c;
        { c = 5; }
        C(){ super(); c = c + 1; }
        int hook(){ return 7; }
    }
    public static void main(String[] x){
        B b = new B(); C c = new C();
        System.out.println(""+b.a+"/"+b.b+"/"+c.a+"/"+c.c);
    }
}
