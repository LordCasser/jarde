package methodthrows;

public abstract class MethodThrowsBoundary {
    public abstract <X extends Exception> void raise() throws X;
}
