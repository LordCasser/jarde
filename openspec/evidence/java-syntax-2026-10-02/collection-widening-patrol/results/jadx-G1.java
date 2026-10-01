
/* JADX INFO: loaded from: G1.class */
public class G1 {
    static <T> T pick(T t, T t2) {
        return t;
    }

    static <T extends java.lang.Comparable<T>> T max(java.util.List<T> list) {
        T t = list.get(0);
        for (T t2 : list) {
            if (t2.compareTo(t) > 0) {
                t = t2;
            }
        }
        return t;
    }

    public static java.lang.String use() {
        java.util.ArrayList arrayList = new java.util.ArrayList();
        arrayList.add("pear");
        arrayList.add("apple");
        arrayList.add("zeta");
        return ((java.lang.String) max(arrayList)) + ":" + ((java.lang.String) pick("a", "b"));
    }

    public static int autoboxLoop() {
        java.lang.Integer numValueOf = 0;
        for (java.lang.Integer numValueOf2 = 1; numValueOf2.intValue() <= 3; numValueOf2 = java.lang.Integer.valueOf(numValueOf2.intValue() + 1)) {
            numValueOf = java.lang.Integer.valueOf(numValueOf.intValue() + numValueOf2.intValue());
        }
        return numValueOf.intValue();
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(use());
        java.lang.System.out.println(autoboxLoop());
    }
}
