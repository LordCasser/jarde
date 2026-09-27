package doublecase;

/* JADX INFO: loaded from: inputs.jar:doublecase/Subject.class */
public final class Subject {
    public static int select(Count count, Animal animal) {
        int i = 0;
        switch (count) {
            case ONE:
                i = 1;
                break;
            case TWO:
                i = 2;
                break;
        }
        switch (animal) {
            case CAT:
                i += 10;
                break;
            case DOG:
                i += 20;
                break;
        }
        return i;
    }
}
