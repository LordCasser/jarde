package p;

public enum Weird$Name {
    FIRST {
        @Override
        public int value() { return 10; }
    },
    SECOND {
        @Override
        public int value() { return 20; }
    };

    public abstract int value();
}
