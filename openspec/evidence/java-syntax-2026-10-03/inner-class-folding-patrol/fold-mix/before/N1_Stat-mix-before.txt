// jarde: presentation of `N1$Stat` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
class N1$Stat extends java.lang.Object {
    N1$Stat() {
        // @method <init>()V
        // @declaration a constructor of `N1$Stat`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    int m() {
        // @method m()I
        // @declaration an instance method of `N1$Stat`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return 1;
    }

    int use(N1 arg1) {
        // jarde: not recovered: the recovery run for `use(LN1;)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method use(LN1;)I
        // @declaration an instance method of `N1$Stat`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 0
        // the instruction at BCI 0 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 3
        // the instruction at BCI 3 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 5
        // the instruction at BCI 5 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6 9 4
        // the copy at BCI 5 has no proved local assignment
        // @bytecode 18 15 12 4
        // the copy at BCI 3 has no proved local assignment
    }
}
