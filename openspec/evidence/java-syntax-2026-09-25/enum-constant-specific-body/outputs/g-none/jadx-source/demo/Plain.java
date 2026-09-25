package demo;

/* JADX INFO: loaded from: input-g-none.jar:demo/Plain.class */
public enum Plain {
    READY,
    WAITING;

    public String label() {
        return name().toLowerCase();
    }
}
