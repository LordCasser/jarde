package methodthrows;

public final class MethodThrowsReflect {
    public static void main(String[] args) throws Exception {
        System.out.println("throws=" + MethodThrowsBoundary.class
                .getDeclaredMethod("raise").getGenericExceptionTypes()[0].getTypeName());
    }
}
