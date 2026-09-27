package single;

/* JADX INFO: loaded from: inputs.jar:single/Subject.class */
public final class Subject {
    public static int select(Mode mode) {
        switch (mode) {
            case ONE:
                return 1;
            case TWO:
                return 2;
            default:
                return 0;
        }
    }
}
