package p;
public class NarrowException {
    private byte value;
    private int accept(byte value) { return value; }
    public int call(boolean flag) {
        try { return accept(flag ? 0 : value); }
        catch (RuntimeException ignored) { return -1; }
    }
}
