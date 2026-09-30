

/* JADX INFO: loaded from: Tf4CleanupProbe.class */
public class Tf4CleanupProbe {
    public static boolean failCall = false;
    public static boolean failCleanup = false;
    public int result = 0;

    public String test() {
        boolean z = false;
        try {
            String strCall = call();
            this.result++;
            z = true;
            if (1 == 0) {
                this.result -= 2;
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
            return strCall;
        } catch (Throwable th) {
            if (!z) {
                this.result -= 2;
                if (failCleanup) {
                    throw new RuntimeException("cleanup");
                }
            }
            throw th;
        }
    }

    private String call() {
        if (failCall) {
            throw new RuntimeException("call");
        }
        return "call";
    }
}
