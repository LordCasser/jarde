package cf08nested;

/* JADX INFO: loaded from: input.jar:cf08nested/NestedEffectful.class */
public final class NestedEffectful {
    static int calls;

    static int cost(int i) {
        calls++;
        return i;
    }

    public static int pick(int[] iArr) {
        int iCost;
        int i;
        if (iArr == null) {
            i = -1;
        } else {
            int i2 = 0;
            while (true) {
                if (i2 >= iArr.length) {
                    iCost = cost(7);
                    break;
                }
                iCost = iArr[i2];
                if (iCost == 3) {
                    break;
                }
                i2++;
            }
            i = iCost + calls;
        }
        return i;
    }
}
