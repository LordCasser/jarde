final class OwnerRefs {
    int value;
    int read() { return value; }
    Runnable handle() { return this::run; }
    void run() { }
    static OwnerRefs make() { return new OwnerRefs(); }
}
