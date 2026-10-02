// jarde: presentation of `VSib` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class VSib extends java.lang.Object {
    static Kid kid;

    VSib() {
        // @method <init>()V
        // @declaration a constructor of `VSib`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Base make() throws Err {
        // @method make()LVSib$Base;
        // @declaration a static method of `VSib`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new Kid();
    }

    public static void main(java.lang.String[] a) throws Err {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VSib`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(make().id());
        return;
    }

    static class Kid extends Base {
        Kid() {
            // @method <init>()V
            // @declaration a constructor of `VSib$Kid`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }

    static class Base extends java.lang.Object {
        Base() {
            // @method <init>()V
            // @declaration a constructor of `VSib$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int id() {
            // @method id()I
            // @declaration an instance method of `VSib$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 3;
        }
    }

    static class Err extends java.lang.Exception {
        Err() {
            // @method <init>()V
            // @declaration a constructor of `VSib$Err`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }
}
