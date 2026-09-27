// jarde: presentation of `em24/Numbers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em24;

public final class Numbers extends java.lang.Object {
    private Numbers() {
        // @method <init>()V
        // @declaration a constructor of `em24.Numbers`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static byte[] bytes() {
        // @method bytes()[B
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new byte[]{0, -1, -10, -1, -128, 127};
    }

    public static short[] shorts() {
        // @method shorts()[S
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new short[]{0, -1, -10, -1, -32768, 32767};
    }

    public static int[] ints() {
        // @method ints()[I
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[]{0, -1, -10, -1, -2147483648, 2147483647};
    }

    public static long[] longs() {
        // @method longs()[J
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new long[]{0L, -1L, -10L, -1L, -9223372036854775808L, 9223372036854775807L};
    }

    public static float[] floats() {
        // @method floats()[F
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new float[]{0x1.19999ap-1f, -0x0.000000p-126f, 0x0.000000p-126f / 0x0.000000p-126f, -0x1.000000p0f / 0x0.000000p-126f, 0x1.000000p0f / 0x0.000000p-126f, 0x0.000002p-126f, 0x1.000000p-126f, 0x1.fffffep127f};
    }

    public static double[] doubles() {
        // @method doubles()[D
        // @declaration a static method of `em24.Numbers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new double[]{0x1.199999999999ap-1d, -0x0.0000000000000p-1022d, 0x0.0000000000000p-1022d / 0x0.0000000000000p-1022d, -0x1.0000000000000p0d / 0x0.0000000000000p-1022d, 0x1.0000000000000p0d / 0x0.0000000000000p-1022d, 0x0.0000000000001p-1022d, 0x1.0000000000000p-1022d, 0x1.fffffffffffffp1023d};
    }
}
