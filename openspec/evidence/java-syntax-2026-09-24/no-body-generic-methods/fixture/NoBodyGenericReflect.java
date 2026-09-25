package nobodygeneric;

public final class NoBodyGenericReflect {
    public static void main(String[] args) throws Exception {
        for (String name : new String[] {"echo", "checked"}) {
            java.lang.reflect.Method method = NoBodyGenericIdentity.class
                    .getDeclaredMethod(name, Number.class);
            System.out.println(name + "=" + method.getTypeParameters().length + ","
                    + method.getGenericReturnType().getTypeName());
        }
        System.out.println("throws=" + NoBodyGenericIdentity.class
                .getDeclaredMethod("checked", Number.class)
                .getGenericExceptionTypes()[0].getTypeName());
    }
}
