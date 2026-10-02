// jarde: presentation of `VGrand` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class VGrand extends java.lang.Object {
    VGrand() {
        // @method <init>()V
        // @declaration a constructor of `VGrand`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VGrand`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new Base().b());
        return;
    }

    static class Base extends java.lang.Object {
        Base() {
            // @method <init>()V
            // @declaration a constructor of `VGrand$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int b() {
            // @method b()I
            // @declaration an instance method of `VGrand$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 6;
        }
    }

    static class Inner extends java.lang.Object {
        Inner() {
            // @method <init>()V
            // @declaration a constructor of `VGrand$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }
}
