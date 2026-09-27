package cf08twolvl;

/* JADX INFO: loaded from: input.jar:cf08twolvl/TwoLevelIf.class */
public final class TwoLevelIf {
    private static int calls;

    private static int cost(int i) {
        calls++;
        return i;
    }

    static int calls() {
        return calls;
    }

    static void resetCalls() {
        calls = 0;
    }

    public static int pick(int[] iArr) {
        int iCost;
        int i;
        if (iArr == null) {
            i = -1;
        } else {
            int length = iArr.length;
            if (length == 0) {
                iCost = -1;
            } else {
                int i2 = 0;
                while (true) {
                    if (i2 >= length) {
                        iCost = cost(7);
                        break;
                    }
                    iCost = iArr[i2];
                    if (iCost == 3) {
                        break;
                    }
                    i2++;
                }
            }
            i = iCost + calls;
        }
        return i;
    }
}
