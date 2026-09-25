package defpackage;

/* JADX INFO: loaded from: MultiCatchProbe.class */
public final class MultiCatchProbe {
    static int finallyCalls;

    static String choosePlain(int i) {
        try {
            if (i == 1) {
                throw new IllegalArgumentException("a");
            }
            if (i == 2) {
                throw new IllegalStateException("s");
            }
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException e) {
            return e.getClass().getSimpleName() + ":" + e.getMessage();
        }
    }

    static String chooseFinally(int i) {
        try {
            if (i == 1) {
                throw new IllegalArgumentException("a");
            }
            if (i == 2) {
                throw new IllegalStateException("s");
            }
            finallyCalls++;
            return "ok";
        } catch (IllegalArgumentException | IllegalStateException e) {
            String str = e.getClass().getSimpleName() + ":" + e.getMessage();
            int i2 = finallyCalls;
            return str;
        } finally {
            finallyCalls++;
        }
    }
}
