package defpackage;

/* JADX INFO: loaded from: CatchAfterFieldStore.class */
public final class CatchAfterFieldStore {
    private static int calls;

    private static void fail() {
        throw new IllegalArgumentException("probe");
    }

    public static int fieldStore(boolean z) {
        calls = 7;
        if (z) {
            try {
                fail();
            } catch (IllegalArgumentException e) {
                return calls + 11;
            }
        }
        return calls;
    }

    public static int localStore(boolean z) {
        if (z) {
            try {
                fail();
            } catch (IllegalArgumentException e) {
                return 7 + 11;
            }
        }
        return 7;
    }
}
