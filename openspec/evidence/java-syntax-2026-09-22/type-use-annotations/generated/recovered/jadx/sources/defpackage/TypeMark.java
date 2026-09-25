package defpackage;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/* JADX INFO: loaded from: TypeMark.class */
@Target({ElementType.TYPE_USE})
@Retention(RetentionPolicy.RUNTIME)
@interface TypeMark {
    String value();
}
