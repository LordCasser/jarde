public class CleanupBoundariesRunner {
    public static void main(String[] args) {
        CleanupBoundaries.trace = 0;
        CleanupBoundaries.value = 0;
        CleanupBoundaries.failAtOne = false;
        try {
            int got = CleanupBoundaries.snapshotReturn();
            System.out.println("snapshot:return=" + got + ":field=" + CleanupBoundaries.value + ":trace=" + CleanupBoundaries.trace);
        } catch (RuntimeException e) {
            System.out.println("snapshot:throw=" + e + ":field=" + CleanupBoundaries.value + ":trace=" + CleanupBoundaries.trace);
        }

        CleanupBoundaries.trace = 0;
        CleanupBoundaries.failAtOne = true;
        try {
            int got = CleanupBoundaries.snapshotReturn();
            System.out.println("snapshot-throw:returned=" + got + ":trace=" + CleanupBoundaries.trace);
        } catch (RuntimeException e) {
            System.out.println("snapshot-throw:" + e.getClass().getName() + ":same=" + (e == CleanupBoundaries.TRY_FAILURE) + ":trace=" + CleanupBoundaries.trace);
        }

        CleanupBoundaries.trace = 0;
        CleanupBoundaries.failAtOne = false;
        try {
            System.out.println("cleanup-return:returned=" + CleanupBoundaries.cleanupThrowsOverReturn() + ":trace=" + CleanupBoundaries.trace);
        } catch (RuntimeException e) {
            System.out.println("cleanup-return:throw=" + e.getClass().getName() + ":cleanup=" + (e == CleanupBoundaries.CLEANUP_FAILURE) + ":try=" + (e == CleanupBoundaries.TRY_FAILURE) + ":trace=" + CleanupBoundaries.trace);
        }

        CleanupBoundaries.trace = 0;
        try {
            System.out.println("cleanup-try-throw:returned=" + CleanupBoundaries.cleanupThrowsOverTryThrow() + ":trace=" + CleanupBoundaries.trace);
        } catch (RuntimeException e) {
            System.out.println("cleanup-try-throw:throw=" + e.getClass().getName() + ":cleanup=" + (e == CleanupBoundaries.CLEANUP_FAILURE) + ":try=" + (e == CleanupBoundaries.TRY_FAILURE) + ":trace=" + CleanupBoundaries.trace);
        }
    }
}
