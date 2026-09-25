package defpackage;

import java.lang.reflect.Field;
import java.lang.reflect.Method;

/* JADX INFO: loaded from: MemberRunner.class */
public class MemberRunner {
    public static void main(String[] strArr) throws Exception {
        Field field = MemberTagged.class.getDeclaredFields()[0];
        Method method = MemberTagged.class.getDeclaredMethods()[0];
        System.out.println(field.isAnnotationPresent(Deprecated.class));
        System.out.println(method.isAnnotationPresent(Deprecated.class));
        System.out.println(method.getParameterAnnotations()[0].length);
        System.out.println(new MemberTagged().value(2));
    }
}
