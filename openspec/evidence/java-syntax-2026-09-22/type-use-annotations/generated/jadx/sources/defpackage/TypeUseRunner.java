package defpackage;

import java.lang.reflect.AnnotatedType;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.Arrays;

/* JADX INFO: loaded from: TypeUseRunner.class */
final class TypeUseRunner {
    TypeUseRunner() {
    }

    private static String value(AnnotatedType annotatedType) {
        TypeMark typeMark = (TypeMark) annotatedType.getAnnotation(TypeMark.class);
        return typeMark == null ? "null" : typeMark.value();
    }

    public static void main(String[] strArr) throws Exception {
        Field declaredField = TypeUseSubject.class.getDeclaredField("field");
        Method declaredMethod = TypeUseSubject.class.getDeclaredMethod("value", new Class[0]);
        Method declaredMethod2 = TypeUseSubject.class.getDeclaredMethod("echo", String.class);
        System.out.println(value(declaredField.getAnnotatedType()));
        System.out.println(value(declaredMethod.getAnnotatedReturnType()));
        System.out.println(value(declaredMethod2.getAnnotatedParameterTypes()[0]));
        System.out.println(Arrays.toString(new TypeUseSubject().echo("ok").toCharArray()));
    }
}
