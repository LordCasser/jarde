package em19;

/* JADX INFO: loaded from: input.jar:em19/Access.class */
public class Access {
    public static int at(int i) {
        return new int[]{1, 2, 3, 5}[i];
    }

    public static int dimensions(int i) {
        return new int[i][i + 1].length;
    }

    public static int[] reverseNegate(int[] iArr) {
        int length = iArr.length;
        int[] iArr2 = new int[length];
        int i = 0;
        int i2 = length;
        while (i2 != 0) {
            i2--;
            int i3 = -iArr[i];
            i++;
            iArr2[i2] = i3 * 5;
        }
        return iArr2;
    }
}
