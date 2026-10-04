public class P07_inner_outer {
    private int v = 7;
    class In { int get(){ return v; } void set(int x){ v = x; } }
    static class St { int use(P07_inner_outer o){ return o.new In().get(); } }
    public static void main(String[] a){ P07_inner_outer o=new P07_inner_outer(); o.new In().set(9); System.out.println(o.v+"/"+new St().use(o)); }
}
