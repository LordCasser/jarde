package bodythrows;

public class BodyThrowsReflect {
    public static void main(String[] args) throws Exception {
        System.out.println("throws=" + BodyThrows.class.getMethod("run")
            .getGenericExceptionTypes()[0].getTypeName());
    }
}
