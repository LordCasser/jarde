package em02;

/* JADX INFO: loaded from: fixture.jar:em02/Contract.class */
public interface Contract {
    int value();

    default int plusOne() {
        return value() + 1;
    }

    static int five() {
        return 5;
    }

    int alsoValue();
}
