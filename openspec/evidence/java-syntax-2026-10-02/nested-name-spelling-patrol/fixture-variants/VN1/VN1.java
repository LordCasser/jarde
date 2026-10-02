public class VN1 {
    static Other.Inner box = new Other.Inner();
    static int use(Other.Inner inner, int v) {
        Object o = new Other.Inner();
        boolean is = o instanceof Other.Inner;
        int viaCast = ((Other.Inner) o).id(v);
        int viaQualifier = Other.Inner.twice(v);
        if (is) {
            return inner.id(v) + viaCast;
        }
        return viaQualifier;
    }
    public static void main(String[] a) {
        System.out.println(use(new Other.Inner(), 21));
    }
}
