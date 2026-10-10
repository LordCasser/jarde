package cf07;

/* JADX INFO: loaded from: LoopCases.class.jar:cf07/LoopCases.class */
public class LoopCases {
    public static int andWhile(boolean z) {
        int i = 0;
        while (z && i < 10) {
            i++;
        }
        return i;
    }

    public static int counted(int i, int i2) {
        int i3 = i + i2;
        int i4 = i;
        while (i4 < i2) {
            i3 = i4 == 7 ? i3 + 2 : i3 * 2;
            i4++;
        }
        return i3 - 1;
    }

    public static int lastIndexOf(int[] iArr, int i, int i2, int i3) {
        for (int i4 = i3 - 1; i4 >= i2; i4--) {
            if (iArr[i4] == i) {
                return i4;
            }
        }
        return -1;
    }
}
