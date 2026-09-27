package single;
public final class Subject {
    public static int select(Mode mode) {
        switch (mode) {
            case ONE: return 1;
            case TWO: return 2;
            default: return 0;
        }
    }
}
