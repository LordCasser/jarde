public class Runner {
    public static void main(String[] args) throws Exception {
        Class<?> probeClass = Class.forName(args[0]);
        java.lang.reflect.Field failCall = probeClass.getField("failCall");
        boolean hasCleanup = args.length > 1;
        int[][] paths = hasCleanup
            ? new int[][] {{0, 0}, {1, 0}, {1, 1}}
            : new int[][] {{0, 0}, {1, 0}};
        for (int[] path : paths) {
            Object probe = probeClass.getConstructor().newInstance();
            failCall.set(null, path[0] == 1);
            if (hasCleanup) {
                probeClass.getField("failCleanup").set(null, path[1] == 1);
            }
            String outcome = "ok";
            String value = null;
            try {
                value = (String) probeClass.getMethod("test").invoke(probe);
            } catch (java.lang.reflect.InvocationTargetException wrapped) {
                Throwable cause = wrapped.getCause();
                outcome = cause.getClass().getSimpleName() + ":" + cause.getMessage();
            }
            int result = (Integer) probeClass.getField("result").get(probe);
            System.out.println("failCall=" + (path[0] == 1) + " failCleanup=" + (path[1] == 1)
                + " outcome=" + outcome + " value=" + value + " result=" + result);
        }
    }
}
