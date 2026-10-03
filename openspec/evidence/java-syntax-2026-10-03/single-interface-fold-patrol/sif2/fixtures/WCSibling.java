// Variant 2 (sibling interfaces): two static interface children called side by side, so two
// distinct `InterfaceMethodRef` owners must anchor in one body.
public class WCSibling {
    interface A { static int sv() { return 8; } }
    interface B { static int sv() { return 9; } }
    public static void main(String[] a) { System.out.println(A.sv() + B.sv()); }
}
