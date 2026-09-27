package em02;
public interface Contract {
    int value();
    default int plusOne() { return value() + 1; }
    static int five() { return 5; }
    abstract int alsoValue();
}
