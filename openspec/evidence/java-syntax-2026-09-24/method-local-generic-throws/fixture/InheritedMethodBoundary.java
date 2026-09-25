package methodthrows;

public abstract class InheritedMethodBoundary extends GenericParent {
    public abstract <X extends Exception> void raise() throws X;
}
