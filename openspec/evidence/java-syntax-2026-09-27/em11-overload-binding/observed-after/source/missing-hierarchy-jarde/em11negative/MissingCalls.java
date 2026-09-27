// jarde: presentation of `em11negative/MissingCalls` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em11negative;

public class MissingCalls extends java.lang.Object {
    public MissingCalls() {
        // @method <init>()V
        // @declaration a constructor of `em11negative.MissingCalls`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String take(em11negative.MissingBase arg0) {
        // @method take(Lem11negative/MissingBase;)Ljava/lang/String;
        // @declaration a static method of `em11negative.MissingCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "base";
    }

    public static java.lang.String take(em11negative.MissingChild arg0) {
        // @method take(Lem11negative/MissingChild;)Ljava/lang/String;
        // @declaration a static method of `em11negative.MissingCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return "child";
    }

    public static java.lang.String run() {
        // jarde: not recovered: the recovery run for `run()Ljava/lang/String;` produced no statement (explanation only); the artifact's own comment lines are below
        // @method run()Ljava/lang/String;
        // @declaration a static method of `em11negative.MissingCalls`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 10 7
        // the parameter 0 of the invocation at BCI 7 is declared `em11negative.MissingBase` presents `em11negative.MissingChild` but the invocation requires `em11negative.MissingBase` and this layer has no safe reference conversion evidence
    }
}
