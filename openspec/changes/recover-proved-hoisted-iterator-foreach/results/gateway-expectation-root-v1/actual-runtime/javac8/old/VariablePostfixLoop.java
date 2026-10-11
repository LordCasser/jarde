// jarde: presentation of `VariablePostfixLoop` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class VariablePostfixLoop extends java.lang.Object {
    public VariablePostfixLoop() {
        // @method <init>()V
        // @declaration a constructor of `VariablePostfixLoop`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `countEmpty(Ljava/util/List;)I`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public static int countEmpty(java.util.List arg0) {
        // @method countEmpty(Ljava/util/List;)I
        // @declaration a static method of `VariablePostfixLoop`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        local1 = 0;
        if (arg0 != null) {
            java.util.Iterator local2;
            local2 = arg0.iterator();
            while (local2.hasNext()) {
                java.lang.String local3 = (java.lang.String) local2.next();
                if (local3.isEmpty()) {
                    local1 = local1 + 1;
                }
            }
        }
        return local1;
    }
}
