package negative;

public class GenericThrowsChild extends GenericThrows {
    @Override
    public <Y extends Exception> void run() throws Y {
        int body = 2;
        if (body != 2) throw new AssertionError();
    }
}
