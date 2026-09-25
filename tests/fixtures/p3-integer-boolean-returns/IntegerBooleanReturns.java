public final class IntegerBooleanReturns {
    public int value;
    public int calls;

    public static int direct(int value) { return value; }
    public static int next(IntegerBooleanReturns self, int value) { self.calls++; return value; }
    public static int once(IntegerBooleanReturns self, int value) { return next(self, value); }
    public int post() { return value++; }
    public int pre() { return ++value; }
    public static synchronized int sync(IntegerBooleanReturns self, int value) { return value; }
    public static int syncOn(Object lock, int value) { synchronized (lock) { return value; } }
}
