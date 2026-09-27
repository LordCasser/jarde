// jarde: presentation of `ExceptionRegionsAudit` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class ExceptionRegionsAudit extends java.lang.Object {
    private static int effects;

    public ExceptionRegionsAudit() {
        // @method <init>()V
        // @declaration a constructor of `ExceptionRegionsAudit`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    private static int work(int arg0) {
        // @method work(I)I
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        ExceptionRegionsAudit.effects = ExceptionRegionsAudit.effects + 1;
        if (arg0 == 0) {
            throw new java.lang.NumberFormatException("zero");
        } else if (arg0 == 2) {
            throw new java.lang.IllegalStateException("two");
    } else {
            return arg0 * 3;
    }
    }

    private static int run() {
        // @method run()I
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        int local1;
        ExceptionRegionsAudit.effects = 0;
        local0 = 0;
        for (local1 = -1; local1 < 4; local1 = local1 + 1) {
            try {
                if (local1 < 0) {
                    ExceptionRegionsAudit.effects = ExceptionRegionsAudit.effects + 1;
                    continue;
                } else {
                    try {
                        local0 = local0 + work(local1);
                    } catch (java.lang.NumberFormatException local2) {
                        ExceptionRegionsAudit.effects = ExceptionRegionsAudit.effects + 10;
                        if (local1 == 0) {
                            continue;
                        } else {
                            local0 = local0 - 1;
                        }
                    }
                    if (local1 == 1) {
                        local0 = local0 + 100;
                    }
                    local0 = local0 + 5;
                    continue;
                }
            } catch (java.lang.IllegalStateException local2) {
                ExceptionRegionsAudit.effects = ExceptionRegionsAudit.effects + 100;
                local0 = local0 + 2;
            }
        }
        return local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ExceptionRegionsAudit`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new java.lang.StringBuilder().append(run()).append(":").append(ExceptionRegionsAudit.effects).toString());
        return;
    }
}
