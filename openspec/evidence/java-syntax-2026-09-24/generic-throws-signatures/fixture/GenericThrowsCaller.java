package genericthrows;

import java.lang.reflect.Method;

public final class GenericThrowsCaller {
    static final class RuntimeCase extends GenericThrowsBoundary<RuntimeException> {
        @Override
        public void invoke() throws RuntimeException {
            System.out.println("invoked");
        }
    }

    static void narrowed(GenericThrowsBoundary<RuntimeException> value) {
        value.invoke();
    }

    public static void main(String[] args) throws Exception {
        GenericThrowsBoundary<RuntimeException> value = new RuntimeCase();
        narrowed(value);
        Method method = GenericThrowsBoundary.class.getMethod("invoke");
        System.out.println("throws=" + method.getGenericExceptionTypes()[0].getTypeName());
    }
}
