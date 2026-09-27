package defpackage;

/* JADX INFO: loaded from: input.jar:ExceptionRegionsAudit.class */
public final class ExceptionRegionsAudit {
    private static int effects;

    private static int work(int i) {
        effects++;
        if (i == 0) {
            throw new NumberFormatException("zero");
        }
        if (i == 2) {
            throw new IllegalStateException("two");
        }
        return i * 3;
    }

    private static int run() {
        effects = 0;
        int iWork = 0;
        for (int i = -1; i < 4; i++) {
            if (i < 0) {
                try {
                    effects++;
                } catch (IllegalStateException e) {
                    effects += 100;
                    iWork += 2;
                }
            } else {
                try {
                    iWork += work(i);
                } catch (NumberFormatException e2) {
                    effects += 10;
                    if (i != 0) {
                        iWork--;
                    }
                }
                if (i == 1) {
                    iWork += 100;
                }
                iWork += 5;
            }
        }
        return iWork;
    }

    public static void main(String[] strArr) {
        System.out.println(run() + ":" + effects);
    }
}
