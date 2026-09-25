package methodthrows;

public final class MethodThrowsRuntime {
    public static void main(String[] args) {
        MethodThrowsCaller.narrowed(new MethodThrowsCaller.RuntimeCase());
    }
}
