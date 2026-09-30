

/* JADX INFO: loaded from: Tf4Probe.class */
public class Tf4Probe {
    public static boolean failCall = false;
    public int result = 0;

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
        if (failCall) {
            throw new RuntimeException("call");
        }
        return "call";
    }
}
