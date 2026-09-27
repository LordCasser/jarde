package grouped;
public final class Subject {
    public static int trace;

    private static int mark(int value) {
        trace = trace * 10 + value;
        return value;
    }

    public static int select(Count count, Animal animal) {
        int result;
        switch (count) {
            case ONE: result = mark(1); break;
            case TWO: result = mark(2); break;
            default: result = mark(3); break;
        }
        switch (animal) {
            case CAT: result += mark(4); break;
            case DOG: result += mark(5); break;
            default: result += mark(6); break;
        }
        return result * 100 + trace;
    }
}
