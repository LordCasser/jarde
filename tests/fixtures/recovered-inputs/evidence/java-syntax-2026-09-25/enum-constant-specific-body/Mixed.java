package demo;

public enum Mixed {
    SPECIAL { @Override public int value() { return 7; } },
    PLAIN;

    public int value() {
        return 0;
    }
}
