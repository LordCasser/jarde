package genericthrows;

public abstract class GenericThrowsBoundary<E extends Exception> {
    public abstract void invoke() throws E;
}
