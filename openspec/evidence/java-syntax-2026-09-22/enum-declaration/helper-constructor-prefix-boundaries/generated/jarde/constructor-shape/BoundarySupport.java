// jarde: presentation of `BoundarySupport` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class BoundarySupport extends java.lang.Object {
    static int calls;

    BoundarySupport() {
        // @method <init>()V
        // @declaration a constructor of `BoundarySupport`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static void constructorEffect() {
        // @method constructorEffect()V
        // @declaration a static method of `BoundarySupport`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        BoundarySupport.calls = BoundarySupport.calls + 1;
        return;
    }

    static int prefixValue(int arg0) {
        // @method prefixValue(I)I
        // @declaration a static method of `BoundarySupport`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        BoundarySupport.calls = BoundarySupport.calls + 1;
        return arg0;
    }
}
