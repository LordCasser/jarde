package doublecase;
public final class Subject {
    public static int select(Count count, Animal animal) {
        int result = 0;
        switch (count) {
            case ONE: result = 1; break;
            case TWO: result = 2; break;
        }
        switch (animal) {
            case CAT: result += 10; break;
            case DOG: result += 20; break;
        }
        return result;
    }
}
