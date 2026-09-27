package dt31;

/* JADX INFO: loaded from: inputs.jar:dt31/SparseSwitch.class */
public final class SparseSwitch {

    /* JADX INFO: loaded from: inputs.jar:dt31/SparseSwitch$Count.class */
    public enum Count {
        ONE,
        TWO,
        THREE
    }

    public static int select(Count count) {
        switch (count.ordinal()) {
            case 1:
                return 1;
            case 2:
                return 2;
            default:
                return 0;
        }
    }
}
