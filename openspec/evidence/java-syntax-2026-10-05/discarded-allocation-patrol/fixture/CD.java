public class CD {
    static class A { int fa = init("fa-A", 1); A(){ this(10); System.out.println("A()"); } A(int x){ System.out.println("A(int):"+x); } static int init(String n,int v){ System.out.println("init:"+n); return v; } }
    static class B extends A { int fb = init("fb-B", 2); B(){ super(20); System.out.println("B()"); } B(int z){ super(21); System.out.println("B(int):"+z); } }
    static class C extends B { int fc = init("fc-C", 3); C(){ this(30); System.out.println("C()"); } C(int y){ super(40); System.out.println("C(int):"+y); } }
    static class D { int fd; D(int v){ fd = v; } D(){ this(7); fd += 1; } }   // this() 委派 + 后置语句
    public static void main(String[] a){ new C(); System.out.println(""+new D().fd); }
}
