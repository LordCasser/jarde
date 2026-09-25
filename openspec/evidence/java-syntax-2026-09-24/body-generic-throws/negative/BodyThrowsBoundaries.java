import java.io.IOException;

class HasCall<E extends Exception> {
    void callee() throws E { }
    void run() throws E { callee(); }
}

class HasHandler<E extends Exception> {
    void run() throws E {
        try {
            risky();
        } catch (IOException ignored) {
            // Real handler and caught exception local.
        }
    }

    private static void risky() throws IOException { throw new IOException(); }
}

class HasThrow<E extends Exception> {
    void run(E value) throws E { throw value; }
}

class GenericParent<T extends Exception> {
    void inherited() throws T { }
}

class InheritedCall<E extends Exception> extends GenericParent<E> {
    void run() throws E { inherited(); }
}

class BodyThrowsBoundaryRunner {
    public static void main(String[] args) {
        new HasCall<RuntimeException>().run();
        new InheritedCall<RuntimeException>().run();
        new HasHandler<RuntimeException>().run();
        try {
            new HasThrow<RuntimeException>().run(new RuntimeException("E"));
        } catch (RuntimeException expected) { }
        System.out.println("verified");
    }
}
