// jarde: presentation of `ByteArrayReturn` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ByteArrayReturn extends java.lang.Object {
    public ByteArrayReturn() {
        // @method <init>()V
        // @declaration a constructor of `ByteArrayReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public byte[] test() {
        // @method test()[B
        // @declaration an instance method of `ByteArrayReturn`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new byte[]{0, 1, 2};
    }
}
