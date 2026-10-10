package defpackage;

/* JADX INFO: loaded from: PrimitiveArrayBranches.jar:PrimitiveArrayBranches.class */
public class PrimitiveArrayBranches {
    private static Object test4(int i) {
        if (i == 1) {
            return new int[]{1, 2};
        }
        if (i == 2) {
            return new float[]{1.0f, 2.0f};
        }
        if (i == 3) {
            return new short[]{1, 2};
        }
        if (i == 4) {
            return new byte[]{1, 2};
        }
        return null;
    }

    public static Object choose(int i) {
        return test4(i);
    }
}
