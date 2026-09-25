public class SwitchRunner {
    private interface Task {
        Object run();
    }

    private static void one(int tag, int mode) {
        SwitchSupport.trace = 0;
        SwitchSupport.mode = mode;
        try {
            Object result = SwitchDeferred.run(tag);
            System.out.println(tag + ":" + mode + ":value=" + result + ":trace=" + SwitchSupport.trace);
        } catch (RuntimeException error) {
            System.out.println(tag + ":" + mode + ":error=" + error.getClass().getName()
                    + ":same=" + (error == SwitchSupport.FAILURE)
                    + ":trace=" + SwitchSupport.trace);
        }
    }

    public static void main(String[] args) {
        for (int tag : new int[] {1, 7}) {
            for (int mode = 0; mode < 3; mode++) {
                one(tag, mode);
            }
        }
    }
}
