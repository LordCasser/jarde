// jarde: presentation of `cf05/ConversionBasic` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf05;

public class ConversionBasic extends java.lang.Object {
    public ConversionBasic() {
        // @method <init>()V
        // @declaration a constructor of `cf05.ConversionBasic`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static int asInt(boolean arg0) {
        // @method asInt(Z)I
        // @declaration a static method of `cf05.ConversionBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 1 : 0;
    }

    public static long asLong(boolean arg0) {
        // @method asLong(Z)J
        // @declaration a static method of `cf05.ConversionBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 1L : 0L;
    }

    public static byte asByte(boolean arg0) {
        // @method asByte(Z)B
        // @declaration a static method of `cf05.ConversionBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (byte) (arg0 ? 1 : 0);
    }

    public static float asFloat(boolean arg0) {
        // @method asFloat(Z)F
        // @declaration a static method of `cf05.ConversionBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 0x1.000000p0f : 0x0.000000p-126f;
    }

    public static double asDouble(boolean arg0) {
        // @method asDouble(Z)D
        // @declaration a static method of `cf05.ConversionBasic`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 0x1.0000000000000p0d : 0x0.0000000000000p-1022d;
    }
}
