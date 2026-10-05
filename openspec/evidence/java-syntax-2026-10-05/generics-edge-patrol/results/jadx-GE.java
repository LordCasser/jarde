package defpackage;

/* JADX INFO: loaded from: GE.class */
public class GE {
    static <T extends java.io.Serializable & java.lang.Comparable<T>> T both(T t, T t2) {
        return ((java.lang.Comparable) t).compareTo(t2) >= 0 ? t : t2;
    }

    static java.lang.String useBoth() {
        return (java.lang.String) both("a", "b");
    }

    static java.util.List<? extends java.lang.Number> up() {
        return java.util.Arrays.asList(1, 2L);
    }

    static int readUp(java.util.List<? extends java.lang.Number> list) {
        return list.get(0).intValue();
    }

    static void writeDown(java.util.List<? super java.lang.Integer> list) {
        list.add(1);
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + useBoth() + "/" + readUp(up()) + "/" + downCheck());
    }

    static int downCheck() {
        java.util.ArrayList arrayList = new java.util.ArrayList();
        writeDown(arrayList);
        return ((java.lang.Integer) arrayList.get(0)).intValue();
    }
}
