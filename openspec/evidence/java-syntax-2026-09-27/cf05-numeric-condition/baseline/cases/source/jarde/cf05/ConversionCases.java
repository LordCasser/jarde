// jarde: presentation of `cf05/ConversionCases` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf05;

public class ConversionCases extends java.lang.Object {
    private byte myByte;

    private short myShort;

    public ConversionCases() {
        // @method <init>()V
        // @declaration a constructor of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.myByte = 7;
        this.myShort = 9;
        return;
    }

    public static int asInt(boolean arg0) {
        // @method asInt(Z)I
        // @declaration a static method of `cf05.ConversionCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 1 : 0;
    }

    public static long asLong(boolean arg0) {
        // @method asLong(Z)J
        // @declaration a static method of `cf05.ConversionCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 1L : 0L;
    }

    public static byte asByte(boolean arg0) {
        // @method asByte(Z)B
        // @declaration a static method of `cf05.ConversionCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return (byte) (arg0 ? 1 : 0);
    }

    public static float asFloat(boolean arg0) {
        // @method asFloat(Z)F
        // @declaration a static method of `cf05.ConversionCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 0x1.000000p0f : 0x0.000000p-126f;
    }

    public static double asDouble(boolean arg0) {
        // @method asDouble(Z)D
        // @declaration a static method of `cf05.ConversionCases`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0 ? 0x1.0000000000000p0d : 0x0.0000000000000p-1022d;
    }

    public int castByte(boolean arg1) {
        // @method castByte(Z)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.write(arg1 ? (byte) 0 : (byte) 1);
    }

    public int byteField(boolean arg1) {
        // jarde: not recovered: the recovery run for `byteField(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method byteField(Z)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 5 9 2
        // the two values joined at BCI 13 do not have a conditional Java type this run can prove
        // @bytecode 0 13
        // the dependency chain from BCI 13 to final consumer 16 is not bounded
        // @bytecode 16 13 0
        // the value at BCI 16 was produced by a saved declaration this run could not commit
    }

    public int castShort(boolean arg1) {
        // jarde: not recovered: the recovery run for `castShort(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method castShort(Z)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 13 10 0
        // the parameter 0 of the invocation at BCI 10 is declared `short` presents `int` and this layer has no proven conversion to `short`
    }

    public int shortField(boolean arg1) {
        // jarde: not recovered: the recovery run for `shortField(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method shortField(Z)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 5 12 2
        // the two values joined at BCI 13 do not have a conditional Java type this run can prove
        // @bytecode 0 13
        // the dependency chain from BCI 13 to final consumer 16 is not bounded
        // @bytecode 16 13 0
        // the value at BCI 16 was produced by a saved declaration this run could not commit
    }

    public int shortConstant(boolean arg1) {
        // jarde: not recovered: the recovery run for `shortConstant(Z)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method shortConstant(Z)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 15 12 0
        // the parameter 0 of the invocation at BCI 12 is declared `short` presents `int` and this layer has no proven conversion to `short`
    }

    private int write(byte arg1) {
        // @method write(B)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return 100 + arg1;
    }

    private int write(short arg1) {
        // @method write(S)I
        // @declaration an instance method of `cf05.ConversionCases`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return 200 + arg1;
    }
}
