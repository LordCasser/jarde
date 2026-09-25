package defpackage;

/* JADX INFO: loaded from: CatchAfterFieldStore.class */
public final class CatchAfterFieldStore {
    private static int calls;

    private static void fail() {
        throw new IllegalArgumentException("probe");
    }

    private static void maybeFail(boolean z) {
        if (z) {
            fail();
        }
    }

    public static int fieldStore(boolean z) {
        calls = 7;
        try {
            maybeFail(z);
            return calls;
        } catch (IllegalArgumentException e) {
            return calls + 11;
        }
    }

    public static int localStore(boolean z) {
        try {
            maybeFail(z);
            return 7;
        } catch (IllegalArgumentException e) {
            return 7 + 11;
        }
    }
}
