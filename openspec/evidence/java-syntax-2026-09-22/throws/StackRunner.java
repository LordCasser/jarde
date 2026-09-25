public class StackRunner {
    public static void main(String[] args) {
        RuntimeException expected = new IllegalArgumentException();
        try { ThrowStack.probe(expected); }
        catch (Throwable actual) {
            System.out.println("identity=" + (actual == expected) + ":calls=" + ThrowStack.calls);
        }
    }
}
