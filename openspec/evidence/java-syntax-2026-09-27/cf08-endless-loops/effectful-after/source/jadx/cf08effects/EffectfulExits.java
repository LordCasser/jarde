package cf08effects;

/* JADX INFO: loaded from: input.jar:cf08effects/EffectfulExits.class */
public final class EffectfulExits {
    static int calls;

    static int cost(int i) {
        calls++;
        return i;
    }

    public static int pick(int[] iArr) {
        int iCost;
        for (int i = 0; i < iArr.length; i++) {
            iCost = iArr[i];
            if (iCost == 3) {
                return iCost + calls;
            }
        }
        iCost = cost(7);
        return iCost + calls;
    }
}
