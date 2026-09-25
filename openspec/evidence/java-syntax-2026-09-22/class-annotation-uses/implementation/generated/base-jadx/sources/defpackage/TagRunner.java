package defpackage;

import java.lang.annotation.Retention;

/* JADX INFO: loaded from: TagRunner.class */
public class TagRunner {
    public static void main(String[] strArr) {
        System.out.println(Tagged.class.isAnnotationPresent(Deprecated.class));
        System.out.println(Tagged.value());
        System.out.println(RetentionTagged.class.getAnnotation(Retention.class));
    }
}
