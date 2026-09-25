package defpackage;

/* JADX INFO: loaded from: CatchAfterFieldAssignment.class */
public final class CatchAfterFieldAssignment {
    private static int field;

    private static void fail() {
        throw new IllegalArgumentException("probe");
    }

    private static void maybeFail(boolean z) {
        if (z) {
            fail();
        }
    }

    public static int fieldAssignment(boolean z) {
        field = 7;
        try {
            maybeFail(z);
        } catch (IllegalArgumentException e) {
            field = 18;
        }
        return field;
    }

    public static int localAssignment(boolean z) {
        int i = 7;
        try {
            maybeFail(z);
        } catch (IllegalArgumentException e) {
            i = 18;
        }
        return i;
    }
}
