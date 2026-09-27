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
        return new java.lang.StringBuilder().append((java.lang.String) local0.call(new java.util.ArrayList())).append("/").append((java.lang.String) local0.call((java.util.List) new java.util.ArrayList())).append("/").append((java.lang.String) local0.call((java.lang.String) null)).append("/").append((java.lang.String) local0.call((java.util.List) null)).append("/").append((java.lang.String) local0.call((java.util.ArrayList) null)).append("/").append((java.lang.String) local0.call((java.lang.String) null)).append("/").append((java.lang.String) local0.call((java.util.List) null)).toString();
    }
}
