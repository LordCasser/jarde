// jarde: presentation of `em11/HMid` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11;

public class HMid extends em11.HBase {
    public HMid() {
        // @method <init>()V
        // @declaration a constructor of `em11.HMid`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `call(Ljava/util/List;)Ljava/lang/String;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.String call(java.util.List arg1) {
        // @method call(Ljava/util/List;)Ljava/lang/String;
        // @declaration an instance method of `em11.HMid`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return "mid-List";
    }
}
