package genericctor;

public final class GenericConstructorTypeCheck {
    static void invalidExplicitTypeArgument() {
        new <Integer> GenericConstructor(Double.valueOf(5.0));
    }
}
