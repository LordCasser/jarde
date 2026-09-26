// jarde: presentation of `probe/EnumArityRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package probe;

public final class EnumArityRunner extends java.lang.Object {
    public EnumArityRunner() {
        // @method <init>()V
        // @declaration a constructor of `probe.EnumArityRunner`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `probe.EnumArityRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("empty=").append(probe.Empty.values().length).toString());
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append("one=").append((java.lang.String) probe.One.ONLY.name()).append(":").append(probe.One.ONLY.ordinal()).append("/").append(probe.One.values().length).toString());
        java.lang.System.out.println("four=" + java.util.Arrays.toString((java.lang.Object[]) probe.Four.values()));
        return;
    }
}
