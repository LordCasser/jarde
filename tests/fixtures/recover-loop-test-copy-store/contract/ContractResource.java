public class ContractResource {
    public static int[] values;
    public static int next, throwAt;
    public static RuntimeException openFailure, readFailure, closeFailure;
    public static StringBuilder events;
    public ContractResource(String path) {
        events.append("open;");
        if (openFailure != null) throw openFailure;
    }
    public int read() {
        int index = next++;
        events.append("read").append(index).append(';');
        if (index == throwAt) throw readFailure;
        if (index >= values.length) throw new AssertionError("bounded input exhausted");
        return values[index];
    }
    public void close() {
        events.append("close;");
        if (closeFailure != null) throw closeFailure;
    }
}
