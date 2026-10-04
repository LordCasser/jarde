public class S {
    private int base = 4;
    class Inner { private int tag; Inner(int t){tag=t;} int total(){return base+tag;} }
    static class Stat {
        int exprForm(S outer) { return outer.new Inner(9).total(); }        // 表达式/直返位
        int localForm(S outer) { S.Inner l = outer.new Inner(9); return l.total(); } // 本地声明位
    }
    public static void main(String[] a){ S s=new S(); Stat t=new Stat(); System.out.println(t.exprForm(s)+"/"+t.localForm(s)); }
}
