public final class ExceptionScope {
    private ExceptionScope() {}

    public static int catchOnly(boolean fail) {
        try {
            if (fail) {
                throw null;
            }
            return 1;
        } catch (NullPointerException caught) {
            int local = 2;
            return local;
        }
    }

    public static int assignedAcrossTry(boolean fail) {
        int local;
        try {
            if (fail) {
                throw null;
            }
            local = 3;
        } catch (NullPointerException caught) {
            local = 4;
        }
        return local;
    }

    public static void main(String[] args) {
        if (catchOnly(false) != 1 || catchOnly(true) != 2
                || assignedAcrossTry(false) != 3 || assignedAcrossTry(true) != 4) {
            throw new AssertionError("exception-scope fixture behavior changed");
        }
    }
}
