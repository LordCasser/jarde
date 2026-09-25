// jarde: presentation of `NestedArrayInitializer` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NestedArrayInitializer extends java.lang.Object {
    static int calls;

    static int trace;

    public NestedArrayInitializer() {
        // @method <init>()V
        // @declaration a constructor of `NestedArrayInitializer`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int element(int arg0) {
        // @method element(I)I
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        NestedArrayInitializer.calls = NestedArrayInitializer.calls + 1;
        NestedArrayInitializer.trace = NestedArrayInitializer.trace * 10 + arg0;
        return arg0;
    }

    static int[][] dynamic() {
        // @method dynamic()[[I
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{element(1), element(2)}, new int[]{element(3)}};
    }

    static java.lang.String[][] literal() {
        // @method literal()[[Ljava/lang/String;
        // @declaration a static method of `NestedArrayInitializer`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.String[][]{new java.lang.String[]{"a", "b"}, new java.lang.String[]{"c"}};
    }
}
