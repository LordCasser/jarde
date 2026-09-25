public class MemberRunner {
    public static void main(String[] args) throws Exception {
        Object field = MemberTagged.class.getDeclaredFields()[0];
        Object method = MemberTagged.class.getDeclaredMethods()[0];
        System.out.println(((java.lang.reflect.Field) field).isAnnotationPresent(Deprecated.class));
        System.out.println(((java.lang.reflect.Method) method).isAnnotationPresent(Deprecated.class));
        System.out.println(((java.lang.reflect.Method) method).getParameterAnnotations()[0].length);
        System.out.println(new MemberTagged().value(2));
    }
}
