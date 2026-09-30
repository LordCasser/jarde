// Drives one probe class through the five behavior paths the certificate pins — the cached
// read (the field preset, the body skipped, the cleanup still runs once and the preset value
// is returned), the uncached read (the body assigns the field, the cleanup runs once), the
// refused validate (the early `return null;` runs the cleanup exactly once), a body failure
// (the cleanup runs once and the original exception rethrows), and a failing cleanup (the new
// exception overrides). The flags are set reflectively; the close count is the shared support
// class's own counter, so every recompiled side is compared on the same oracle.
public class Runner {
    public static void main(String[] args) throws Exception {
        Class<?> probeClass = Class.forName(args[0]);
        int[][] paths = { { 0, 0, 0, 1 }, { 0, 0, 0, 0 }, { 1, 0, 0, 0 }, { 0, 1, 0, 0 },
            { 0, 0, 1, 0 } };
        for (int[] path : paths) {
            Support.failValidate = path[0] == 1;
            Support.failBody = path[1] == 1;
            java.lang.reflect.Field failCleanup = probeClass.getField("failCleanup");
            failCleanup.set(null, path[2] == 1);
            Support.closes = 0;
            String outcome = "ok";
            String value = "null";
            try {
                Object probe = probeClass.getConstructor().newInstance();
                if (path[3] == 1) {
                    probe.getClass().getField("bytes").set(probe, new byte[] { 9 });
                }
                Object returned = probeClass.getMethod("test").invoke(probe);
                if (returned instanceof byte[]) {
                    value = "len=" + ((byte[]) returned).length;
                }
            } catch (java.lang.reflect.InvocationTargetException wrapped) {
                Throwable cause = wrapped.getCause();
                outcome = cause.getClass().getSimpleName() + ":" + cause.getMessage();
            }
            System.out.println("failValidate=" + (path[0] == 1) + " failBody=" + (path[1] == 1)
                + " failCleanup=" + (path[2] == 1) + " cached=" + (path[3] == 1) + " outcome="
                + outcome + " value=" + value + " closes=" + Support.closes);
        }
    }
}
