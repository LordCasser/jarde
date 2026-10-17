final class EXExtraExit {
    private static void cleanup() {}

    static String finThrow(int value) {
        try {
            if (value < 0) {
                throw new RuntimeException("negative");
            }
            if (value == 0) {
                return "zero";
            }
            return "ok";
        } finally {
            cleanup();
        }
    }
}
