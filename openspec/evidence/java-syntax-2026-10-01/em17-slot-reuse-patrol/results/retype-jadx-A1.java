
/* JADX INFO: loaded from: A1.class */
public class A1 {
    static int calls = 0;

    static int n() {
        calls++;
        return 3;
    }

    public static int[][] dynDims() {
        int[][] iArr = new int[n()][];
        iArr[0] = new int[n()];
        return new int[][]{iArr[0], new int[n()]};
    }

    public static Object[] mixed() {
        return new Object[]{"s", Integer.valueOf(n()), new int[n()]};
    }

    public static void main(String[] strArr) {
        int[][] iArrDynDims = dynDims();
        System.out.println(iArrDynDims.length + ":" + iArrDynDims[0].length + ":" + calls);
        System.out.println(mixed().length + ":" + calls);
    }
}
