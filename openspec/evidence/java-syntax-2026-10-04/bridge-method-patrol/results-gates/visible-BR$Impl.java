// jarde: presentation of `BR$Impl` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class BR$Impl extends java.lang.Object implements java.lang.Comparable {
    BR$Impl() {
        // @method <init>()V
        // @declaration a constructor of `BR$Impl`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public int compareTo(BR$Impl arg1) {
        // @method compareTo(LBR$Impl;)I
        // @declaration an instance method of `BR$Impl`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return 0;
    }

    public int compareTo(java.lang.Object arg1) {
        // @method compareTo(Ljava/lang/Object;)I
        // @declaration an instance method of `BR$Impl`, member flags 0x1041
        // recovered from bytecode; presentation is not claimed to compile
        return this.compareTo((BR$Impl) arg1);
    }
}
