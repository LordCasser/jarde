package methodthrows;

public final class MethodThrowsCaller {
    static final class RuntimeCase extends MethodThrowsBoundary {
        @Override
        public <X extends Exception> void raise() throws X {
            System.out.println("raised");
        }
    }

    static void narrowed(MethodThrowsBoundary value) {
        value.<RuntimeException>raise();
    }

    public static void main(String[] args) throws Exception {
        narrowed(new RuntimeCase());
        System.out.println("throws=" + MethodThrowsBoundary.class
                .getDeclaredMethod("raise").getGenericExceptionTypes()[0].getTypeName());
    }
}
