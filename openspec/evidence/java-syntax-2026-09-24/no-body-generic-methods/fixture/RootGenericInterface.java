package nobodygeneric;

public interface RootGenericInterface {
    <T extends Number> T echo(T value);

    <X extends Exception> void raise() throws X;
}
