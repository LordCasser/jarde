public class AG {
    static java.util.List list = new java.util.ArrayList<>();
    static int idx = 0;
static int viaArg() {
        AG.list.add((java.lang.Object) "a");
        AG.list.add((java.lang.Object) "b");
        java.util.List saved0 = AG.list;
        return AG.idx;
    }

    public static void main(String[] a){ System.out.println(AG.viaArg()); }
}
