// jarde: presentation of `ConditionalFieldWrites` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ConditionalFieldWrites extends java.lang.Object {
    public static boolean staticFlag;

    public boolean instanceFlag;

    public int integerField;

    public ConditionalFieldWrites() {
        // @method <init>()V
        // @declaration a constructor of `ConditionalFieldWrites`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void putStatic(boolean choose) {
        // @method putStatic(Z)V
        // @declaration a static method of `ConditionalFieldWrites`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalFieldWrites.staticFlag = (choose ? 2 : 3) % 2 != 0;
        return;
    }

    public void putInstance(boolean choose) {
        // @method putInstance(Z)V
        // @declaration an instance method of `ConditionalFieldWrites`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.instanceFlag = (choose ? 2 : 3) % 2 != 0;
        return;
    }

    public void putInteger(boolean choose) {
        // @method putInteger(Z)V
        // @declaration an instance method of `ConditionalFieldWrites`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.integerField = choose ? 2 : 3;
        return;
    }

    public static void main(java.lang.String[] args) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ConditionalFieldWrites`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        ConditionalFieldWrites value = new ConditionalFieldWrites();
        putStatic(false);
        value.putInstance(true);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(ConditionalFieldWrites.staticFlag).append(":").append(value.instanceFlag).toString());
        putStatic(true);
        value.putInstance(false);
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(ConditionalFieldWrites.staticFlag).append(":").append(value.instanceFlag).toString());
        return;
    }
}
