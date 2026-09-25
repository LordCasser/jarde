@interface FloatingArrays {
    float[] floats() default {0.5f, -0.0f};
    double[] doubles() default {0x0.0000000000001p-1022d, -1.0d};
}
