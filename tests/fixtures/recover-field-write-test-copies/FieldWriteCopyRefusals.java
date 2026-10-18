public final class FieldWriteCopyRefusals {
    public volatile int z;
    public boolean flag;
    public int calls;
    public int throwAt = -1;
    public final RuntimeException marker = new RuntimeException("field-copy-refusal-marker");

    public FieldWriteCopyRefusals() {
    }

    private int next(int value) {
        calls++;
        if (calls == throwAt) {
            throw marker;
        }
        return value + 1;
    }

    public boolean differentReceiver(FieldWriteCopyRefusals receiver, int value) {
        return (receiver.z = receiver.next(value)) < 5;
    }

    public boolean nonIntField(boolean value, boolean threshold) {
        return (this.flag = value) == threshold;
    }

    public boolean handledAssignment(int value) {
        try {
            return (this.z = next(value)) < 5;
        } catch (RuntimeException failure) {
            this.z = -99;
            return false;
        }
    }
}
