package nobodygeneric;

public final class RootGenericReflect {
    public static void main(String[] args) throws Exception {
        System.out.println("echo=" + RootGenericInterface.class
                .getDeclaredMethod("echo", Number.class).getTypeParameters().length);
        System.out.println("raise=" + RootGenericInterface.class
                .getDeclaredMethod("raise").getTypeParameters().length);
        System.out.println("throws=" + RootGenericInterface.class
                .getDeclaredMethod("raise").getGenericExceptionTypes()[0].getTypeName());
    }
}
