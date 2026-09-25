// jarde: presentation of `BasicRunner` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class BasicRunner extends java.lang.Object {
    BasicRunner() {
        // @method <init>()V
        // @declaration a constructor of `BasicRunner`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) throws java.lang.Exception {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `BasicRunner`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.Object) Basic.class.getMethod("count", new java.lang.Class[0]).getDefaultValue());
        java.lang.System.out.println((java.lang.Object) Basic.class.getMethod("label", new java.lang.Class[0]).getDefaultValue());
        java.lang.System.out.println((java.lang.String) java.util.Arrays.toString((int[]) Basic.class.getMethod("codes", new java.lang.Class[0]).getDefaultValue()));
        return;
    }
}
