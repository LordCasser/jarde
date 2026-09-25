package negative;

public class GenericThrowsCore {
    public <X extends Exception> void run() throws X {
        int body = 1;
        if (body != 1) throw new AssertionError();
    }
}
