// jarde: presentation of `em11/InputHierarchyCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11;

public class InputHierarchyCalls extends java.lang.Object {
    public InputHierarchyCalls() {
        // @method <init>()V
        // @declaration a constructor of `em11.InputHierarchyCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String take(em11.HMid arg0) {
        // @method take(Lem11/HMid;)Ljava/lang/String;
        // @declaration a static method of `em11.InputHierarchyCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "mid";
    }

    public static java.lang.String take(em11.HLeaf arg0) {
        // @method take(Lem11/HLeaf;)Ljava/lang/String;
        // @declaration a static method of `em11.InputHierarchyCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "leaf";
    }

    public static java.lang.String run() {
        // @method run()Ljava/lang/String;
        // @declaration a static method of `em11.InputHierarchyCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return take((em11.HMid) new em11.HLeaf());
    }
}
