package nobodygeneric;

public final class RootGenericCaller {
    static final class Impl implements RootGenericInterface {
        @Override
        public <T extends Number> T echo(T value) {
            return value;
        }

        @Override
        public <X extends Exception> void raise() throws X {
            System.out.println("raised");
        }
    }

    static void narrowed(RootGenericInterface value) {
        value.<RuntimeException>raise();
    }

    public static void main(String[] args) throws Exception {
        RootGenericInterface value = new Impl();
        Integer result = value.echo(Integer.valueOf(9));
        narrowed(value);
        System.out.println("value=" + result);
        System.out.println("returns=" + RootGenericInterface.class
                .getDeclaredMethod("echo", Number.class).getGenericReturnType().getTypeName());
        System.out.println("throws=" + RootGenericInterface.class
                .getDeclaredMethod("raise").getGenericExceptionTypes()[0].getTypeName());
    }
}
