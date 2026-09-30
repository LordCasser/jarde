package defpackage;

/* JADX INFO: loaded from: Tf4.class */
public class Tf4 {
    private int result = 0;

    public String test() {
        boolean z = false;
        try {
            String strCall = call();
            this.result++;
            boolean z2 = true;
            return strCall;
        } finally {
            if (!z) {
                this.result -= 2;
            }
        }
    }

    private String call() {
        return "call";
    }

    public int check() {
        test();
        return this.result;
    }
}
