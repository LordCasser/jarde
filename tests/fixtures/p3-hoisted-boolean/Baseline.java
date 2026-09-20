// The baseline driver of the hoisted-boolean fixture: the committed original class is run in a
// controlled way, and this is what its own bytecode answers — the values the executed comparison's
// generated side must return for the same calls. `copied` and `swapped` are called with both values
// of `b` and with `n` on either side of the branch (`0` takes the arm the source's `then` writes,
// `7` and `-1` the other), so both writes of `c` are executed on each side; `relayed` runs the copy
// chain in both arms, and the controls keep the inputs the default set would pick (`README.md`
// records the run).
public class Baseline {
    public static void main(String[] args) {
        System.out.println("copied(true, 0)=" + HoistedBoolean.copied(true, 0));
        System.out.println("copied(false, 0)=" + HoistedBoolean.copied(false, 0));
        System.out.println("copied(true, 7)=" + HoistedBoolean.copied(true, 7));
        System.out.println("copied(false, -1)=" + HoistedBoolean.copied(false, -1));
        System.out.println("swapped(true, 0)=" + HoistedBoolean.swapped(true, 0));
        System.out.println("swapped(false, 0)=" + HoistedBoolean.swapped(false, 0));
        System.out.println("swapped(true, 7)=" + HoistedBoolean.swapped(true, 7));
        System.out.println("swapped(false, -1)=" + HoistedBoolean.swapped(false, -1));
        System.out.println("relayed(true)=" + HoistedBoolean.relayed(true));
        System.out.println("relayed(false)=" + HoistedBoolean.relayed(false));
        System.out.println("literalArmed(true)=" + HoistedBoolean.literalArmed(true));
        System.out.println("literalArmed(false)=" + HoistedBoolean.literalArmed(false));
        System.out.println("fromParameter(true, 0)=" + HoistedBoolean.fromParameter(true, 0));
        System.out.println("fromParameter(false, 7)=" + HoistedBoolean.fromParameter(false, 7));
        System.out.println("fromParameter(true, -1)=" + HoistedBoolean.fromParameter(true, -1));
        System.out.println("intLocal(7)=" + HoistedBoolean.intLocal(7));
        System.out.println("intLocal(0)=" + HoistedBoolean.intLocal(0));
        System.out.println("intLocal(-1)=" + HoistedBoolean.intLocal(-1));
    }
}
