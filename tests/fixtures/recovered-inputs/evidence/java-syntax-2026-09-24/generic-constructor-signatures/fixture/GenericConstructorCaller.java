package genericctor;

public final class GenericConstructorCaller {
    public static void main(String[] args) throws Exception {
        GenericConstructor value = new <Integer> GenericConstructor(Integer.valueOf(5));
        System.out.println("class=" + value.getClass().getName());
        System.out.println("types=" + GenericConstructor.class
                .getDeclaredConstructor(Number.class).getTypeParameters()[0].getName());
        System.out.println("parameter=" + GenericConstructor.class
                .getDeclaredConstructor(Number.class).getGenericParameterTypes()[0].getTypeName());
    }
}
