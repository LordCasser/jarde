// jarde: presentation of `BR$Base` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class BR$Base extends java.lang.Object implements BR$Node {
    BR$Base() {
        // @method <init>()V
        // @declaration a constructor of `BR$Base`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public BR$Base next() {
        // @method next()LBR$Base;
        // @declaration an instance method of `BR$Base`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return new BR$Base();
    }

    // jarde: projected bridge `next()LBR$Node;` at physical method record 2: bridge@1 proved a pure call at BCI 1 to source declaration `next()LBR$Base;` at method record 1; the resolved erased contract lets javac regenerate the bridge
}
