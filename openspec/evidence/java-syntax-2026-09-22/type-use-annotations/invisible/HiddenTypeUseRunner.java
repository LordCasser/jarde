import java.lang.reflect.AnnotatedType;

final class HiddenTypeUseRunner {
    public static void main(String[] args) throws Exception {
        AnnotatedType field = HiddenTypeUse.class.getDeclaredField("field").getAnnotatedType();
        AnnotatedType returned = HiddenTypeUse.class.getDeclaredMethod("value").getAnnotatedReturnType();
        AnnotatedType parameter = HiddenTypeUse.class.getDeclaredMethod("echo", String.class)
                .getAnnotatedParameterTypes()[0];
        System.out.println(field.getAnnotations().length + "," + returned.getAnnotations().length
                + "," + parameter.getAnnotations().length);
    }
}
