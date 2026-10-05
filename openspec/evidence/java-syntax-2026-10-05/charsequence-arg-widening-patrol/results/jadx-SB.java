package defpackage;

/* JADX INFO: loaded from: SB.class */
public class SB {
    static java.lang.String acc(java.util.List<java.lang.String> list) {
        java.lang.StringBuilder sb = new java.lang.StringBuilder();
        java.util.Iterator<java.lang.String> it = list.iterator();
        while (it.hasNext()) {
            sb.append(it.next()).append(',');
        }
        return sb.toString();
    }

    static java.lang.String cond(java.util.List<java.lang.String> list) {
        java.lang.String str = "";
        for (java.lang.String str2 : list) {
            if (str2.length() > 1) {
                str = str + str2;
            }
        }
        return str;
    }

    static int[] copy(int[] iArr) {
        int[] iArr2 = new int[iArr.length];
        java.lang.System.arraycopy(iArr, 0, iArr2, 0, iArr.length);
        return iArr2;
    }

    static int[] of(int[] iArr) {
        return java.util.Arrays.copyOf(iArr, iArr.length + 1);
    }

    static java.lang.String join(java.lang.String[] strArr) {
        return java.lang.String.join("-", strArr);
    }

    static int spread(int... iArr) {
        int i = 0;
        for (int i2 : iArr) {
            i += i2;
        }
        return i;
    }

    static int call() {
        return spread(1, 2, 3);
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println(acc(java.util.Arrays.asList("a", "b")) + "/" + cond(java.util.Arrays.asList("x", "yy")) + "/" + java.util.Arrays.toString(copy(new int[]{1, 2})) + "/" + java.util.Arrays.toString(of(new int[]{1})) + "/" + join(new java.lang.String[]{"p", "q"}) + "/" + spread(4, 5) + "/" + call());
    }
}
