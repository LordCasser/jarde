public class SynchronizedMultiExitRunner {
    private static void run(String name, boolean first, int failId) {
        Object lock = new Object();
        SynchronizedMultiExit.trace = 0;
        SynchronizedMultiExit.failId = failId;
        try {
            int result = SynchronizedMultiExit.choose(lock, first);
            System.out.println(name + ":return=" + result + ":trace=" + SynchronizedMultiExit.trace);
        } catch (RuntimeException e) {
            System.out.println(name + ":throw=" + e.getClass().getName() + ":same=" + (e == SynchronizedMultiExit.FAILURE) + ":trace=" + SynchronizedMultiExit.trace);
        }
    }
    public static void main(String[] args) {
        run("first", true, 0);
        run("second", false, 0);
        run("first-throws", true, 1);
    }
}
