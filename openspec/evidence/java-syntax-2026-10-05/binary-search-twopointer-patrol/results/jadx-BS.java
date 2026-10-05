package defpackage;

/* JADX INFO: loaded from: BS.class */
public class BS {
    static java.util.Map<java.lang.Integer, java.lang.Integer> memo = new java.util.HashMap();

    static int bsearch(int[] iArr, int i) {
        int i2 = 0;
        int length = iArr.length - 1;
        while (i2 <= length) {
            int i3 = (i2 + length) >>> 1;
            int i4 = iArr[i3];
            if (i4 < i) {
                i2 = i3 + 1;
            } else {
                if (i4 <= i) {
                    return i3;
                }
                length = i3 - 1;
            }
        }
        return -(i2 + 1);
    }

    static int[] twoPtr(int[] iArr) {
        int i = 0;
        for (int i2 = 0; i2 < iArr.length; i2++) {
            if (iArr[i2] != 0) {
                int i3 = i;
                i++;
                iArr[i3] = iArr[i2];
            }
        }
        return java.util.Arrays.copyOf(iArr, i);
    }

    static int fib(int i) {
        if (i < 2) {
            return i;
        }
        java.lang.Integer num = memo.get(java.lang.Integer.valueOf(i));
        if (num != null) {
            return num.intValue();
        }
        int iFib = fib(i - 1) + fib(i - 2);
        memo.put(java.lang.Integer.valueOf(i), java.lang.Integer.valueOf(iFib));
        return iFib;
    }

    public static void main(java.lang.String[] strArr) {
        java.lang.System.out.println("" + bsearch(new int[]{1, 3, 5, 7}, 5) + "/" + bsearch(new int[]{1, 3}, 4) + "/" + java.util.Arrays.toString(twoPtr(new int[]{0, 1, 0, 2, 3, 0})) + "/" + fib(40));
    }
}
