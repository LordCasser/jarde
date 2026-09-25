package demo;

public enum Plain {
    READY,
    WAITING;

    public String label() {
        return name().toLowerCase();
    }
}
