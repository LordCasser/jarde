package dt31;

public class SparseSwitch {

    public enum Count {
        ONE,
        TWO,
        THREE
    }

    public static int select(Count value) {
        switch (value.ordinal()) {
            case 1:
                return 1;
            case 2:
                return 2;
            default:
                return 0;
        }
    }
}
