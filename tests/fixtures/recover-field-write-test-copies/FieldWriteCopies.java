public final class FieldWriteCopies {
    public volatile int z;
    public int calls;
    public long ledger;
    public int throwAt = -1;
    public final RuntimeException marker = new RuntimeException("field-copy-marker");
    public int lastValue;

    public FieldWriteCopies() {
    }

    private int next(int value) {
        calls++;
        lastValue = value;
        ledger = ledger * 37L + value;
        if (calls == throwAt) {
            throw marker;
        }
        return value + 1;
    }

    public boolean assignAndCompare(int value) {
        return (this.z = next(value)) < 5;
    }

    public int statefulWhile(int start) {
        int iterations = 0;
        while ((this.z = next(start + iterations)) < 3) {
            iterations++;
        }
        return iterations;
    }
}
