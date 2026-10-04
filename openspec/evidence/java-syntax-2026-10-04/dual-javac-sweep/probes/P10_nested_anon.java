public class P10_nested_anon {
    interface O { String s(); }
    static O mk(){ return new O(){ public String s(){ return new O(){ public String s(){ return "deep"; } }.s(); } }; }
    public static void main(String[] a){ System.out.println(mk().s()); }
}
