// jarde: presentation of `cf11/HoistedIteratorRefusal` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf11;

public final class HoistedIteratorRefusal extends java.lang.Object {
    public HoistedIteratorRefusal() {
        // @method <init>()V
        // @declaration a constructor of `cf11.HoistedIteratorRefusal`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sumWithWrittenGap(Ljava/util/List;)I`: unsupported (ordinary_generic_source_unproved): method body or no-body declaration has no complete source proof
    public static int sumWithWrittenGap(java.util.List values) {
        // jarde: not recovered: the recovery run for `sumWithWrittenGap(Ljava/util/List;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method sumWithWrittenGap(Ljava/util/List;)I
        // @declaration a static method of `cf11.HoistedIteratorRefusal`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0 13 22 44 47 50
        // local 1 crosses a quoted fallback region; its assignments and consumers cannot be presented as one lexically bound definition-use slice
    }
}
