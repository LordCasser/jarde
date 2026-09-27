// jarde: presentation of `em11/HierarchyCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11;

public class HierarchyCalls extends java.lang.Object {
    public HierarchyCalls() {
        // @method <init>()V
        // @declaration a constructor of `em11.HierarchyCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String run() {
        // @method run()Ljava/lang/String;
        // @declaration a static method of `em11.HierarchyCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        em11.HLeaf local0 = new em11.HLeaf();
        // @bytecode 128 125 122 111 106 98 93 82 77 66 61 50 45 31 26 23 42 58 55 74 71 90 87 103 119 116 15 34 53 69 85 101 114
        // the parameter 0 of the invocation at BCI 42 is declared `java.util.List` presents `java.util.ArrayList` but the invocation requires `java.util.List` and this layer has no safe reference conversion evidence
    }
}
