// jarde: presentation of `em02/other/CrossPackageChild` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em02.other;

public class CrossPackageChild extends em02.PackageBase {
    public CrossPackageChild() {
        // @method <init>()V
        // @declaration a constructor of `em02.other.CrossPackageChild`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void onlyHere() {
        // @method onlyHere()V
        // @declaration an instance method of `em02.other.CrossPackageChild`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.stamp = 3;
        return;
    }
}
