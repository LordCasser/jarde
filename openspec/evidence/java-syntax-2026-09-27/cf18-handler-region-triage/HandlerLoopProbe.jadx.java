
/* JADX INFO: loaded from: input.jar:HandlerLoopProbe.class */
public final class HandlerLoopProbe {
    private static int effects;

    private static int work(int i) {
        if (i == 0) {
            throw new NumberFormatException("zero");
        }
        if (i == 2) {
            throw new IllegalStateException("two");
        }
        return i;
    }

    private static int run() {
        int iWork = 0;
        for (int i = 0; i < 3; i++) {
            try {
                try {
                    iWork += work(i);
                    iWork++;
                } catch (NumberFormatException e) {
                    effects += 10;
                }
            } catch (IllegalStateException e2) {
                effects += 100;
                iWork += 2;
            }
        }
        return iWork;
    }

    public static void main(String[] strArr) {
        System.out.println(run() + ":" + effects);
    }
}
