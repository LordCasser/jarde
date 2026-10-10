// jarde: presentation of `DependentArrayStores` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class DependentArrayStores extends java.lang.Object {
    public DependentArrayStores() {
        // @method <init>()V
        // @declaration a constructor of `DependentArrayStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int[] test() {
        // @method test()[I
        // @declaration an instance method of `DependentArrayStores`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        int[] local1 = new int[3];
        local1[0] = 1;
        local1[1] = local1[0] + 1;
        local1[2] = local1[1] + 1;
        return local1;
    }
}
