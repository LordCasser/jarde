// jarde: presentation of `UnusedByteArrayInit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class UnusedByteArrayInit extends java.lang.Object {
    public UnusedByteArrayInit() {
        // @method <init>()V
        // @declaration a constructor of `UnusedByteArrayInit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void test() {
        // @method test()V
        // @declaration an instance method of `UnusedByteArrayInit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        byte[] local1 = new byte[]{10, 20, 30};
        return;
    }
}
