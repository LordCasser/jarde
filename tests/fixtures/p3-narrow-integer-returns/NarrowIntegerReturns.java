public class NarrowIntegerReturns {
    public int value;

    public static int directByte(String unused, int value) { return value; }
    public static int directChar(Number unused, int value) { return value; }
    public static int directShort(boolean unused, int value) { return value; }

    public static int byteLocal(byte value) { byte copy = value; return copy; }
    public static int charLocal(char value) { char copy = value; return copy; }
    public static int shortLocal(short value) { short copy = value; return copy; }
    public static int integerControl(byte value, boolean unused) { byte copy = value; return copy; }

    public int postByte(int unused) { return value++; }
    public int preChar(long unused) { return ++value; }
    public int postShort(float unused) { return value++; }

    public static int syncByte(Object lock, int value) {
        synchronized (lock) { return value; }
    }

    public static int syncChar(String lock, int value) {
        synchronized (lock) { return value; }
    }

    public static int syncShort(Object lock, long unused, int value) {
        synchronized (lock) { return value; }
    }
}
