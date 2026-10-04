public class D_covariant {
    static class A { Object get(){ return "a"; } }
    static class B extends A { String get(){ return "b"; } }   // 协变返回 -> bridge
    public static void main(String[] x){ System.out.println(new A().get()+new B().get()); }
}
