public final class AccessorVerify {
    private boolean f;

    private static void access$102(AccessorVerify receiver, boolean value) {
        receiver.f = value;
    }

    public void write(boolean value) {
        access$102(this, value);
    }

    public boolean value() {
        return f;
    }
}
