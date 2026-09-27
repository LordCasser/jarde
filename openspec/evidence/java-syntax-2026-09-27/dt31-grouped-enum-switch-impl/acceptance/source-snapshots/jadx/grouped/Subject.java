package grouped;

/* JADX INFO: loaded from: grouped-input.jar:grouped/Subject.class */
public final class Subject {
    public static int trace;

    private static int mark(int i) {
        trace = (trace * 10) + i;
        return i;
    }

    public static int select(Count count, Animal animal) {
        int iMark;
        int iMark2;
        switch (count) {
            case ONE:
                iMark = mark(1);
                break;
            case TWO:
                iMark = mark(2);
                break;
            default:
                iMark = mark(3);
                break;
        }
        switch (animal) {
            case CAT:
                iMark2 = iMark + mark(4);
                break;
            case DOG:
                iMark2 = iMark + mark(5);
                break;
            default:
                iMark2 = iMark + mark(6);
                break;
        }
        return (iMark2 * 100) + trace;
    }
}
