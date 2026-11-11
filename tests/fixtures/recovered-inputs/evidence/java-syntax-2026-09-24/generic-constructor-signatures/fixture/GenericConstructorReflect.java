package genericctor;

public final class GenericConstructorReflect {
    public static void main(String[] args) throws Exception {
        java.lang.reflect.Constructor<?> ctor = GenericConstructor.class
                .getDeclaredConstructor(Number.class);
        System.out.println("types=" + ctor.getTypeParameters().length);
        System.out.println("parameter=" + ctor.getGenericParameterTypes()[0].getTypeName());
    }
}
