package bodythrows;

public class BodyThrowsCaller {
    static void narrowed(BodyThrows<RuntimeException> value) {
        value.run();
    }

    public static void main(String[] args) throws Exception {
        narrowed(new BodyThrows<RuntimeException>());
        System.out.println("throws=" + BodyThrows.class.getMethod("run")
            .getGenericExceptionTypes()[0].getTypeName());
    }
}
