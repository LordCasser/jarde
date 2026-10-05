public class IC {
    static class Box<T> { T v; Box(T v){ this.v = v; } T get(){ return v; } }
    static Box<String> make(){ return new Box<>("s"); }              // 菱形返回
    static String use(){ return make().get(); }                       // 链式消费
    static class Pair<A,B> { A a; B b; Pair(A a,B b){ this.a=a; this.b=b; } }
    static Pair<String,Integer> nested(){ return new Pair<>("k", 1); } // 双参菱形
    static String takeP(Pair<String,Integer> p){ return p.a + ":" + p.b; }
    static java.util.Map<String, java.util.List<Integer>> deep(){ return new java.util.HashMap<>(); } // 深层泛型菱形
    public static void main(String[] a){ System.out.println(""+use()+"/"+takeP(nested())+"/"+deep().size()); }
}
