// jarde: presentation of `FloatingArrays` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
@interface FloatingArrays {
    // jarde: no body: the member `floats()[F` is declared abstract and its declaration carries no Code attribute
    public abstract float[] floats() default {0x1.000000p-1f, -0.0f};

    // jarde: no body: the member `doubles()[D` is declared abstract and its declaration carries no Code attribute
    public abstract double[] doubles() default {0x0.0000000000001p-1022d, -0x1.0000000000000p0d};
}
