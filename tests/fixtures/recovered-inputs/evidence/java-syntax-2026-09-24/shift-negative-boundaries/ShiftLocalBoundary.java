/** Source-only input for local rewrites; its class file is generated in a temporary directory. */
public final class ShiftLocalBoundary {
    public static int savedBeforeOverwrite(int value, int distance) {
        int saved = value;
        value = value + 1;
        return saved << distance;
    }
}
