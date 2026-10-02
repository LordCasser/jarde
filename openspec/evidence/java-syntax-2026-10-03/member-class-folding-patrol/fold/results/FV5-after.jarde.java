// jarde: presentation of `VGRef` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class VGRef extends java.lang.Object {
    VGRef() {
        // @method <init>()V
        // @declaration a constructor of `VGRef`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static Inner hold(VGRef$Inner$Leaf leaf) {
        // @method hold(LVGRef$Inner$Leaf;)LVGRef$Inner;
        // @declaration a static method of `VGRef`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return null;
    }

    public static void main(java.lang.String[] a) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `VGRef`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new Base().b());
        return;
    }

    static class Base extends java.lang.Object {
        Base() {
            // @method <init>()V
            // @declaration a constructor of `VGRef$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int b() {
            // @method b()I
            // @declaration an instance method of `VGRef$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 8;
        }
    }

    static class Inner extends java.lang.Object {
        Inner() {
            // @method <init>()V
            // @declaration a constructor of `VGRef$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }
}
