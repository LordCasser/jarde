// P3 6.8: an annotation type's members declare their defaults in the class file.
//
//     javac --release 8 -g:none -d v8 Marker.java
//
// Every member below carries an `AnnotationDefault` attribute (`javap -v` shows `s#10`,
// `I#13`, `Z#13`, `J#18`, `C#22`, `e#25.#26`, `[I#29,I#30` and `D#33`). `Kind` is a
// top-level enum in this file, so the same command writes two class files — `Marker.class`
// and `Kind.class` — and the `e` tag's type index resolves in the same pool the class read
// resolves.
public @interface Marker {
    String value() default "x";

    int count() default 1;

    boolean flag() default true;

    long big() default 5L;

    char grade() default 'A';

    Kind kind() default Kind.ONE;

    int[] pair() default {3, 4};

    double ratio() default 0.5;
}

enum Kind {
    ONE,
    TWO
}
