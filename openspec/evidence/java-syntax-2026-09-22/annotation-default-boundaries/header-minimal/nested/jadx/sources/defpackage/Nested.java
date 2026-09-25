package defpackage;

/* JADX INFO: loaded from: Nested.class */
@interface Nested {
    Inner child() default @Inner(6);

    Inner[] children() default {@Inner(2), @Inner(3)};
}
