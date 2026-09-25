package defpackage;

/* JADX INFO: loaded from: LoopTryHandlerEntry.class */
public class LoopTryHandlerEntry {
    static int maybeFail(int i, int i2) {
        if (i == i2) {
            throw new IllegalStateException();
        }
        return i;
    }

    static int loopTry(int i, int i2) {
        int iMaybeFail = 0;
        while (i > 0) {
            try {
                iMaybeFail += maybeFail(i, i2);
            } catch (RuntimeException e) {
                iMaybeFail = -1;
            }
            i--;
        }
        return iMaybeFail;
    }
}
