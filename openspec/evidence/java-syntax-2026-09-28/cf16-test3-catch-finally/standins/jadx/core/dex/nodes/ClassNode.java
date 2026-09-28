package jadx.core.dex.nodes;

public class ClassNode {
    public static final StringBuilder events = new StringBuilder();
    public int loadCount;
    public int unloadCount;
    public Throwable loadFailure;
    public Throwable unloadFailure;

    public static void event(String name) {
        if (events.length() != 0) {
            events.append(',');
        }
        events.append(name);
    }

    public void load() {
        event("load");
        loadCount++;
        fail(loadFailure);
    }

    public void unload() {
        event("unload");
        unloadCount++;
        fail(unloadFailure);
    }

    private static void fail(Throwable failure) {
        if (failure instanceof Error) {
            throw (Error) failure;
        }
        if (failure instanceof RuntimeException) {
            throw (RuntimeException) failure;
        }
    }

    @Override
    public String toString() {
        return "ClassNode";
    }
}
