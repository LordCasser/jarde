// jarde: presentation of `p/Capture$1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package p;

class Capture$1 extends java.lang.Object implements java.lang.Runnable {
    final double val$d;

    // jarde: generic Signature projection refused for `<init>(D)V`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter count
    Capture$1(double arg1) {
        // @method <init>(D)V
        // @declaration a constructor of `p.Capture$1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        this.val$d = arg1;
        super();
        return;
    }

    public void run() {
        // @method run()V
        // @declaration an instance method of `p.Capture$1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(this.val$d);
        return;
    }
}
