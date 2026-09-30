// Drives one probe class through the four behavior paths the certificate pins — normal
// (close once, value returned), body failure after the assignment (close once, the original
// exception rethrown), a null query (no close), and a failing cleanup (the new exception
// overrides). The flags are set reflectively; the close count is the shared support class's
// own counter, so every recompiled side is compared on the same oracle.
public class Runner {
    public static void main(String[] args) throws Exception {
        Class<?> probeClass = Class.forName(args[0]);
        int[][] paths = { { 0, 0, 0 }, { 1, 0, 0 }, { 0, 1, 0 }, { 1, 0, 1 } };
        for (int[] path : paths) {
            Cursor.failBody = path[0] == 1;
            Context.failQuery = path[1] == 1;
            java.lang.reflect.Field failCleanup = probeClass.getField("failCleanup");
            failCleanup.set(null, path[2] == 1);
            Throwables.closes = 0;
            String outcome = "ok";
            String value = null;
            try {
                value = (String) probeClass
                    .getMethod("test", Context.class, Object.class)
                    .invoke(probeClass.getConstructor().newInstance(), new Context(), new Object());
            } catch (java.lang.reflect.InvocationTargetException wrapped) {
                Throwable cause = wrapped.getCause();
                outcome = cause.getClass().getSimpleName() + ":" + cause.getMessage();
            }
            System.out.println("failBody=" + (path[0] == 1) + " failQuery=" + (path[1] == 1)
                + " failCleanup=" + (path[2] == 1) + " outcome=" + outcome + " value=" + value
                + " closes=" + Throwables.closes);
        }
    }
}
