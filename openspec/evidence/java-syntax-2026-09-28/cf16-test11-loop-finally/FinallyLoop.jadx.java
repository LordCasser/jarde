package defpackage;

import java.util.List;

/* JADX INFO: loaded from: FinallyLoop.class */
public class FinallyLoop {
    private int count;
    public static boolean fail;

    public void test(List<Object> list) {
        try {
            call1();
        } finally {
            for (Object item : list) {
                call2(item);
            }
        }
    }

    private void call1() {
        this.count += 100;
        if (fail) {
            throw new IllegalStateException("body");
        }
    }

    private void call2(Object item) {
        this.count++;
    }

    public int count() {
        return this.count;
    }
}
