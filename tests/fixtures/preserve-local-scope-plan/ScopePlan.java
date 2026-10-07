public final class ScopePlan {
    private ScopePlan() {}

    static int catchOnly(boolean fail) {
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

    static int assignedAcrossTry(boolean fail) {
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

    static int assignedAcrossIf(boolean flag) {
        int local;
        if (flag) {
            local = 5;
        } else {
            local = 6;
        }
        return local;
    }

    static int nestedHandlerOnly(Runnable task) {
        try {
            try {
                task.run();
            } catch (IllegalArgumentException inner) {
                int local = 8;
                return local;
            }
        } catch (RuntimeException outer) {
            return 9;
        }
        return 0;
    }

    static int nestedAcross(boolean fail) {
        int local;
        try {
            try {
                if (fail) {
                    throw null;
                }
                local = 10;
            } catch (NullPointerException inner) {
                local = 11;
            }
        } catch (RuntimeException outer) {
            local = 12;
        }
        return local;
    }

    public static void main(String[] args) {
        System.out.print(catchOnly(false) + "," + catchOnly(true) + ",");
        System.out.print(assignedAcrossTry(false) + "," + assignedAcrossTry(true) + ",");
        System.out.print(assignedAcrossIf(false) + "," + assignedAcrossIf(true) + ",");
        System.out.print(nestedHandlerOnly((Runnable) null) + ",");
        System.out.print(nestedAcross(false) + "," + nestedAcross(true) + "\n");
    }
}
