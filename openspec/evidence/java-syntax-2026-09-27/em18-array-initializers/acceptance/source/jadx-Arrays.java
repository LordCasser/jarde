package em18;

/* JADX INFO: loaded from: input.jar:em18/Arrays.class */
public class Arrays {
    public static String[] strings() {
        return new String[]{"1", "2", "3"};
    }

    public static int[] ints(int i) {
        return new int[]{1, i + 1, 2};
    }

    public static int[] postfix(int i) {
        return new int[]{1, i, (i + 1) * 2};
    }

    public static int[] selfRead() {
        int[] iArr = new int[3];
        iArr[0] = 1;
        iArr[1] = iArr[0] + 1;
        iArr[2] = iArr[1] + 1;
        return iArr;
    }

    public static int objectArg(Exception exc) {
        return use(new Object[]{exc});
    }

    private static int use(Object[] objArr) {
        return objArr.length;
    }
}
