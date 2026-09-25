package defpackage;

import java.lang.annotation.Repeatable;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;

/* JADX INFO: loaded from: Tag.class */
@Retention(RetentionPolicy.CLASS)
@Repeatable(Tags.class)
@interface Tag {
    String value();
}
